use std::collections::HashSet;
use std::path::Path;

use polarbear_vocab_application::{ApplicationError, CsvImportPort};
use polarbear_vocab_domain::{
    CsvImportIssue, CsvImportPreview, CsvImportSample, DatasetImportPlan, ImportedSense,
};

#[derive(Clone, Debug, Default)]
pub struct CsvDatasetImporter;

impl CsvImportPort for CsvDatasetImporter {
    fn preview(&self, path: &str) -> Result<CsvImportPreview, ApplicationError> {
        let parsed = parse_file(path)?;
        Ok(CsvImportPreview {
            file_name: file_name(path),
            total_rows: parsed.total_rows,
            valid_rows: parsed.entries.len() as u32,
            issues: parsed.issues,
            sample: parsed
                .entries
                .iter()
                .take(5)
                .map(|entry| CsvImportSample {
                    sense_uid: entry.sense_uid.clone(),
                    lemma: entry.lemma.clone(),
                    part_of_speech: entry.part_of_speech.clone(),
                    quiz_prompt_zh: entry.quiz_prompt_zh.clone(),
                })
                .collect(),
        })
    }

    fn parse(&self, path: &str) -> Result<DatasetImportPlan, ApplicationError> {
        let parsed = parse_file(path)?;
        if !parsed.issues.is_empty() {
            return Err(ApplicationError::InvalidInput(format!(
                "CSV contains {} invalid row(s)",
                parsed.issues.len()
            )));
        }
        if parsed.entries.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "CSV does not contain vocabulary rows".to_owned(),
            ));
        }
        Ok(DatasetImportPlan {
            entries: parsed.entries,
        })
    }
}

struct ParsedCsv {
    entries: Vec<ImportedSense>,
    issues: Vec<CsvImportIssue>,
    total_rows: u32,
}

fn parse_file(path: &str) -> Result<ParsedCsv, ApplicationError> {
    validate_file(path)?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(import_error)?;
    let headers = reader.headers().map_err(import_error)?.clone();
    let columns = Columns::from_headers(&headers)?;
    let mut entries = Vec::new();
    let mut issues = Vec::new();
    let mut seen = HashSet::new();
    for (index, record) in reader.records().enumerate() {
        let row = index as u32 + 2;
        match record
            .map_err(import_error)
            .and_then(|record| columns.entry(&record, row))
        {
            Ok(entry) if seen.insert(entry.sense_uid.clone()) => entries.push(entry),
            Ok(entry) => issues.push(CsvImportIssue {
                row,
                message: format!("duplicate sense_uid: {}", entry.sense_uid),
            }),
            Err(ApplicationError::InvalidInput(message)) => {
                issues.push(CsvImportIssue { row, message });
            }
            Err(error) => return Err(error),
        }
        if index >= 50_000 {
            return Err(ApplicationError::InvalidInput(
                "CSV may contain at most 50,000 rows".to_owned(),
            ));
        }
    }
    Ok(ParsedCsv {
        total_rows: entries.len() as u32 + issues.len() as u32,
        entries,
        issues,
    })
}

fn validate_file(path: &str) -> Result<(), ApplicationError> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        ApplicationError::InvalidInput(format!("cannot read selected CSV: {error}"))
    })?;
    if !metadata.is_file() || metadata.len() > 10 * 1024 * 1024 {
        return Err(ApplicationError::InvalidInput(
            "CSV must be a file no larger than 10 MB".to_owned(),
        ));
    }
    Ok(())
}

struct Columns {
    sense_uid: Option<usize>,
    lemma: usize,
    pos: Option<usize>,
    quiz_prompt_zh: usize,
    gloss_zh: Option<usize>,
    ipa_us: Option<usize>,
    ipa_uk: Option<usize>,
    example_en: Option<usize>,
    example_zh: Option<usize>,
}

impl Columns {
    fn from_headers(headers: &csv::StringRecord) -> Result<Self, ApplicationError> {
        let find = |name: &str| headers.iter().position(|header| header == name);
        Ok(Self {
            sense_uid: find("sense_uid"),
            lemma: find("lemma").ok_or_else(|| missing_column("lemma"))?,
            pos: find("pos"),
            quiz_prompt_zh: find("quiz_prompt_zh")
                .ok_or_else(|| missing_column("quiz_prompt_zh"))?,
            gloss_zh: find("gloss_zh"),
            ipa_us: find("ipa_us"),
            ipa_uk: find("ipa_uk"),
            example_en: find("example_en"),
            example_zh: find("example_zh"),
        })
    }

    fn entry(
        &self,
        record: &csv::StringRecord,
        row: u32,
    ) -> Result<ImportedSense, ApplicationError> {
        let lemma = required(record, self.lemma, "lemma", row)?;
        let prompt = required(record, self.quiz_prompt_zh, "quiz_prompt_zh", row)?;
        let part_of_speech = optional(record, self.pos).unwrap_or("unknown");
        let sense_uid = optional(record, self.sense_uid)
            .map(str::to_owned)
            .unwrap_or_else(|| generated_uid(lemma, part_of_speech));
        validate_length(lemma, "lemma", 100)?;
        validate_length(prompt, "quiz_prompt_zh", 300)?;
        Ok(ImportedSense {
            sense_uid,
            lemma: lemma.to_owned(),
            part_of_speech: part_of_speech.to_owned(),
            quiz_prompt_zh: prompt.to_owned(),
            gloss_zh: optional(record, self.gloss_zh).unwrap_or(prompt).to_owned(),
            ipa_us: optional(record, self.ipa_us).unwrap_or_default().to_owned(),
            ipa_uk: optional(record, self.ipa_uk).unwrap_or_default().to_owned(),
            example_en: optional(record, self.example_en)
                .unwrap_or_default()
                .to_owned(),
            example_zh: optional(record, self.example_zh)
                .unwrap_or_default()
                .to_owned(),
        })
    }
}

fn required<'a>(
    record: &'a csv::StringRecord,
    index: usize,
    name: &str,
    row: u32,
) -> Result<&'a str, ApplicationError> {
    optional(record, Some(index))
        .ok_or_else(|| ApplicationError::InvalidInput(format!("row {row}: {name} is required")))
}

fn optional(record: &csv::StringRecord, index: Option<usize>) -> Option<&str> {
    index
        .and_then(|index| record.get(index))
        .filter(|value| !value.trim().is_empty())
}

fn generated_uid(lemma: &str, part_of_speech: &str) -> String {
    let slug: String = lemma
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect();
    format!(
        "en:{}:{}:1",
        slug.trim_matches('-'),
        part_of_speech.to_lowercase()
    )
}

fn validate_length(value: &str, name: &str, maximum: usize) -> Result<(), ApplicationError> {
    if value.chars().count() > maximum {
        return Err(ApplicationError::InvalidInput(format!(
            "{name} exceeds {maximum} characters"
        )));
    }
    Ok(())
}

fn missing_column(name: &str) -> ApplicationError {
    ApplicationError::InvalidInput(format!("CSV is missing required column: {name}"))
}

fn import_error(error: csv::Error) -> ApplicationError {
    ApplicationError::InvalidInput(format!("invalid CSV: {error}"))
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dataset.csv")
        .to_owned()
}
