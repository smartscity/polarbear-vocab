use polarbear_vocab_application::{ApplicationError, ArticleRepository};
use polarbear_vocab_storage_sqlite::{DatabasePaths, SqliteStore};
use rusqlite::Connection;

#[test]
fn articles_are_persisted_listed_and_deleted() {
    let directory = tempfile::tempdir().unwrap();
    let content_path = directory.path().join("content.db");
    Connection::open(&content_path).unwrap();
    let store = SqliteStore::open(&DatabasePaths {
        content: content_path,
        user: directory.path().join("user.db"),
    })
    .unwrap();

    let saved = store
        .save_article("Listening practice", "The article stays on this device.")
        .unwrap();

    let articles = store.list_articles().unwrap();
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0], saved);
    store.delete_article(&saved.id).unwrap();
    assert!(store.list_articles().unwrap().is_empty());
    assert!(matches!(
        store.delete_article(&saved.id),
        Err(ApplicationError::NotFound(_))
    ));
}
