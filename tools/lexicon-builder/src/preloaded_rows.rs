use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, ensure};

use crate::model::Manifest;

const FILES: [(&str, &str); 6] = [
    ("primary-school", "polarbear-primary-pep.csv"),
    ("junior-high", "polarbear-junior.csv"),
    ("senior-high", "polarbear-high-school.csv"),
    ("cet4", "polarbear-cet4.csv"),
    ("cet6", "polarbear-cet6.csv"),
    ("ielts", "polarbear-ielts.csv"),
];
const HEADERS: [&str; 9] = [
    "sense_uid",
    "lemma",
    "pos",
    "quiz_prompt_zh",
    "gloss_zh",
    "ipa_us",
    "ipa_uk",
    "example_en",
    "example_zh",
];

#[derive(Clone, Debug)]
pub struct Row {
    pub uid: String,
    pub lemma: String,
    pub pos: String,
    pub prompt: String,
    pub gloss: String,
    pub ipa_us: String,
    pub ipa_uk: String,
    pub example_en: String,
    pub example_zh: String,
}

#[derive(Clone, Debug)]
pub struct Membership {
    pub dataset_id: String,
    pub sense_uid: String,
    pub sequence: u32,
}

pub struct Batch {
    pub rows: Vec<Row>,
    pub memberships: Vec<Membership>,
}

pub fn load(directory: &Path, manifest: &Manifest) -> anyhow::Result<Batch> {
    ensure!(
        directory.is_dir(),
        "preloaded dataset directory does not exist"
    );
    let known: HashSet<&str> = manifest
        .datasets
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let mut batch = Batch {
        rows: Vec::new(),
        memberships: Vec::new(),
    };
    let mut indexes = HashMap::<String, usize>::new();
    for (dataset_id, name) in FILES {
        ensure!(
            known.contains(dataset_id),
            "manifest is missing dataset {dataset_id}"
        );
        load_file(directory, dataset_id, name, &mut batch, &mut indexes)?;
    }
    ensure!(
        batch.rows.len() >= 4,
        "preloaded lexicon needs at least four distinct senses"
    );
    Ok(batch)
}

fn load_file(
    directory: &Path,
    dataset_id: &str,
    name: &str,
    batch: &mut Batch,
    indexes: &mut HashMap<String, usize>,
) -> anyhow::Result<()> {
    let path = directory.join(name);
    let metadata = path
        .metadata()
        .with_context(|| format!("cannot read {}", path.display()))?;
    ensure!(
        metadata.is_file() && metadata.len() <= 10 * 1024 * 1024,
        "invalid dataset file {name}"
    );
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&path)?;
    let headers = reader.headers()?;
    ensure!(
        headers.len() == HEADERS.len()
            && headers
                .iter()
                .zip(HEADERS)
                .all(|(actual, expected)| actual.trim_start_matches('\u{feff}') == expected),
        "unexpected CSV columns in {name}"
    );
    let mut seen = HashSet::new();
    let mut count = 0_u32;
    for (index, record) in reader.records().enumerate() {
        ensure!(index < 50_000, "dataset {name} exceeds 50,000 rows");
        let row = parse(&record.with_context(|| format!("{name}: row {}", index + 2))?)?;
        ensure!(
            seen.insert(row.uid.clone()),
            "duplicate sense_uid in {name}: {}",
            row.uid
        );
        batch.memberships.push(Membership {
            dataset_id: dataset_id.to_owned(),
            sense_uid: row.uid.clone(),
            sequence: index as u32,
        });
        if let Some(existing) = indexes.get(&row.uid).copied() {
            ensure!(
                batch.rows[existing].lemma.eq_ignore_ascii_case(&row.lemma)
                    && batch.rows[existing].pos == row.pos,
                "conflicting lemma or POS for {}",
                row.uid
            );
            if quality(&row) > quality(&batch.rows[existing]) {
                batch.rows[existing] = row;
            }
        } else {
            indexes.insert(row.uid.clone(), batch.rows.len());
            batch.rows.push(row);
        }
        count += 1;
    }
    ensure!(count > 0, "dataset {name} is empty");
    Ok(())
}

fn parse(record: &csv::StringRecord) -> anyhow::Result<Row> {
    let field = |index: usize| record.get(index).unwrap_or_default().trim().to_owned();
    let uid = field(0);
    let lemma = field(1);
    let pos = field(2);
    let prompt = field(3);
    ensure!(
        !uid.is_empty() && !lemma.is_empty() && !prompt.is_empty(),
        "missing required vocabulary field"
    );
    ensure!(
        uid.len() <= 200 && lemma.chars().count() <= 100 && prompt.chars().count() <= 300,
        "vocabulary field exceeds the supported length"
    );
    let gloss = field(4);
    Ok(Row {
        uid,
        lemma,
        pos: if pos.is_empty() {
            "unknown".to_owned()
        } else {
            pos
        },
        prompt: prompt.clone(),
        gloss: if gloss.is_empty() { prompt } else { gloss },
        ipa_us: field(5),
        ipa_uk: field(6),
        example_en: field(7),
        example_zh: field(8),
    })
}

fn quality(row: &Row) -> u8 {
    u8::from(!row.ipa_us.is_empty())
        + u8::from(!row.ipa_uk.is_empty())
        + u8::from(!row.example_en.is_empty())
        + 2 * u8::from(row.example_en.ends_with(['.', '!', '?']))
        + u8::from(!row.gloss.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{FILES, HEADERS, Row, load, quality};
    use crate::model::{Dataset, Manifest, Source};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn fixture_directory() -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("polarbear-preloaded-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).expect("create fixture directory");
        path
    }

    fn fixture_manifest() -> Manifest {
        Manifest {
            source: Source {
                name: "test".to_owned(),
                license: "test".to_owned(),
                url: None,
                attribution: "test".to_owned(),
            },
            datasets: FILES
                .iter()
                .map(|(id, _)| Dataset {
                    id: (*id).to_owned(),
                    name: (*id).to_owned(),
                })
                .collect(),
        }
    }

    fn write_fixtures(directory: &Path, duplicate: bool) {
        for (dataset_id, name) in FILES {
            let mut writer = csv::Writer::from_path(directory.join(name)).expect("open fixture");
            writer.write_record(HEADERS).expect("write header");
            let rich = dataset_id == "cet4";
            writer
                .write_record([
                    "shared.v.01",
                    "earn",
                    "verb",
                    "赚得",
                    "获得；赚得",
                    if rich { "/ɜːrn/" } else { "" },
                    "",
                    if rich {
                        "She earned their trust."
                    } else {
                        "earn money"
                    },
                    "",
                ])
                .expect("write shared sense");
            for index in 0..2 {
                let uid = if duplicate && dataset_id == "primary-school" {
                    "shared.v.01".to_owned()
                } else {
                    format!("{dataset_id}.{index}.01")
                };
                writer
                    .write_record([
                        uid.as_str(),
                        "other",
                        "noun",
                        "其他",
                        "其他",
                        "",
                        "",
                        "",
                        "",
                    ])
                    .expect("write unique sense");
            }
            writer.flush().expect("flush fixture");
        }
    }

    #[test]
    fn full_sentence_and_pronunciation_win_over_a_sparse_phrase() {
        let sparse = Row {
            uid: "earn.v.01".to_owned(),
            lemma: "earn".to_owned(),
            pos: "verb".to_owned(),
            prompt: "赚得".to_owned(),
            gloss: "赚得".to_owned(),
            ipa_us: String::new(),
            ipa_uk: String::new(),
            example_en: "earn money".to_owned(),
            example_zh: String::new(),
        };
        let rich = Row {
            ipa_us: "/ɜːrn/".to_owned(),
            example_en: "She earned their trust.".to_owned(),
            ..sparse.clone()
        };
        assert!(quality(&rich) > quality(&sparse));
    }

    #[test]
    fn shared_senses_keep_six_memberships_and_choose_richer_content() {
        let directory = fixture_directory();
        write_fixtures(&directory, false);
        let batch = load(&directory, &fixture_manifest()).expect("load fixture datasets");
        assert_eq!(batch.rows.len(), 13);
        assert_eq!(batch.memberships.len(), 18);
        assert_eq!(
            batch
                .memberships
                .iter()
                .filter(|item| item.sense_uid == "shared.v.01")
                .count(),
            6
        );
        let shared = batch
            .rows
            .iter()
            .find(|item| item.uid == "shared.v.01")
            .expect("shared sense");
        assert_eq!(shared.ipa_us, "/ɜːrn/");
        assert_eq!(shared.example_en, "She earned their trust.");
        fs::remove_dir_all(&directory).expect("remove test fixture");
    }

    #[test]
    fn duplicate_sense_within_one_dataset_is_rejected() {
        let directory = fixture_directory();
        write_fixtures(&directory, true);
        let error = load(&directory, &fixture_manifest())
            .err()
            .expect("reject fixture");
        assert!(error.to_string().contains("duplicate sense_uid"));
        fs::remove_dir_all(&directory).expect("remove test fixture");
    }
}
