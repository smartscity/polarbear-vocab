use polarbear_vocab_domain::BuiltinArticle;

pub fn load() -> Result<Vec<BuiltinArticle>, serde_json::Error> {
    serde_json::from_str(include_str!("../../data/builtin/listening-phrases.json"))
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn bundled_packs_contain_sixty_spoken_sentences() {
        let packs = load().unwrap();
        let sentence_count = packs
            .iter()
            .flat_map(|pack| pack.body.lines())
            .filter(|line| {
                line.trim_start()
                    .starts_with(|character: char| character.is_numeric())
            })
            .count();

        assert_eq!(packs.len(), 4);
        assert_eq!(sentence_count, 60);
    }
}
