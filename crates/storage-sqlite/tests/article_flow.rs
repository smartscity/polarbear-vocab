use polarbear_vocab_application::{ApplicationError, ArticleRepository};
use polarbear_vocab_domain::BuiltinArticle;
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
    store
        .save_article_translation(&saved.id, "文章保存在本机。")
        .unwrap();
    assert_eq!(
        store.list_articles().unwrap()[0].translated_body.as_deref(),
        Some("文章保存在本机。")
    );
    store.delete_article(&saved.id).unwrap();
    assert!(store.list_articles().unwrap().is_empty());
    assert!(matches!(
        store.delete_article(&saved.id),
        Err(ApplicationError::NotFound(_))
    ));
}

#[test]
fn builtin_listening_packs_are_upserted_and_cannot_be_deleted() {
    let directory = tempfile::tempdir().unwrap();
    let content_path = directory.path().join("content.db");
    Connection::open(&content_path).unwrap();
    let store = SqliteStore::open(&DatabasePaths {
        content: content_path,
        user: directory.path().join("user.db"),
    })
    .unwrap();
    let pack = BuiltinArticle {
        id: "builtin.everyday".to_owned(),
        title: "Everyday".to_owned(),
        body: "How are you?".to_owned(),
        translated_body: "你好吗？".to_owned(),
    };

    store.ensure_builtin_articles(&[pack]).unwrap();

    let article = store.list_articles().unwrap().remove(0);
    assert!(article.builtin);
    assert_eq!(article.translated_body.as_deref(), Some("你好吗？"));
    assert!(matches!(
        store.delete_article(&article.id),
        Err(ApplicationError::Conflict(_))
    ));
    assert!(matches!(
        store.save_article_translation(&article.id, "changed"),
        Err(ApplicationError::Conflict(_))
    ));
}
