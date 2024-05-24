mod prepare;

use entity::conversation;
use prepare::prepare_mock_db;
use service::{Mutation, Query};

#[tokio::test]
async fn main() {
    let db = &prepare_mock_db();

    {
        let conversation = Query::find_conversation_by_id(db, 1)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(conversation.id, 1);
    }

    {
        let conversation = Query::find_conversation_by_id(db, 5)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(conversation.id, 5);
    }

    {
        let conversation = Mutation::create_conversation(
            db,
            conversation::Model {
                id: 0,
                title: "Title D".to_owned(),
                updated_at: None,
                created_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            conversation,
            conversation::ActiveModel {
                id: sea_orm::ActiveValue::Unchanged(6),
                title: sea_orm::ActiveValue::Unchanged("Title D".to_owned()),
                updated_at: sea_orm::ActiveValue::Set(conversation.updated_at),
                created_at: sea_orm::ActiveValue::Set(conversation.created_at)
            }
        );
    }

    {
        let conversation = Mutation::update_conversation_by_id(
            db,
            1,
            conversation::Model {
                id: 1,
                title: "New Title A".to_owned(),
                created_at: None,
                updated_at: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            conversation,
            conversation::Model {
                id: 1,
                title: "New Title A".to_owned(),
                updated_at: sea_orm::ActiveValue::Set(conversation.updated_at),
                created_at: sea_orm::ActiveValue::Set(conversation.created_at)
            }
        );
    }

    {
        let result = Mutation::delete_conversation(db, 5).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }

    {
        let result = Mutation::delete_all_conversations(db).await.unwrap();

        assert_eq!(result.rows_affected, 5);
    }
}
