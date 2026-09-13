use polarbear_vocab_domain::CollectionSpec;

pub trait CollectionResolver {
    type Error;

    fn resolve(&self, spec: &CollectionSpec) -> Result<Vec<String>, Self::Error>;
}
