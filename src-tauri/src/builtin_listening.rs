use polarbear_vocab_domain::BuiltinArticle;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioPack {
    id: String,
    title: String,
    heading_en: String,
    heading_zh: String,
    sentences: Vec<[String; 2]>,
}

pub fn load() -> Result<Vec<BuiltinArticle>, serde_json::Error> {
    let mut articles: Vec<BuiltinArticle> =
        serde_json::from_str(include_str!("../../data/builtin/listening-phrases.json"))?;
    let scenarios: Vec<ScenarioPack> =
        serde_json::from_str(include_str!("../../data/builtin/listening-scenes.json"))?;
    articles.extend(scenarios.into_iter().map(scenario_article));
    Ok(articles)
}

fn scenario_article(pack: ScenarioPack) -> BuiltinArticle {
    let mut english = format!("# {}\n\n", pack.heading_en);
    let mut chinese = format!("# {}\n\n", pack.heading_zh);
    for (index, [sentence_en, sentence_zh]) in pack.sentences.iter().enumerate() {
        english.push_str(&format!("{}. {}\n", index + 1, sentence_en));
        chinese.push_str(&format!("{}. {}\n", index + 1, sentence_zh));
    }
    BuiltinArticle {
        id: pack.id,
        title: pack.title,
        body: english.trim_end().to_owned(),
        translated_body: chinese.trim_end().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn bundled_packs_are_bilingual_and_use_stable_ids() {
        let packs = load().unwrap();
        let ids: std::collections::HashSet<_> = packs.iter().map(|pack| &pack.id).collect();
        let sentence_count: usize = packs
            .iter()
            .map(|pack| {
                pack.body
                    .lines()
                    .filter(|line| line.starts_with(|character: char| character.is_ascii_digit()))
                    .count()
            })
            .sum();
        assert_eq!(packs.len(), 16);
        assert_eq!(sentence_count, 267);
        assert_eq!(ids.len(), packs.len());
        assert!(packs.iter().all(|pack| {
            pack.body
                .lines()
                .filter(|line| line.starts_with("1. "))
                .count()
                == 1
                && pack.body.lines().count() == pack.translated_body.lines().count()
        }));
        assert!(
            packs
                .iter()
                .any(|pack| pack.id == "builtin.phrases.developer-interview-technical")
        );
    }
}
