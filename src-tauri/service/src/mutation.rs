use ::entity::{conversation, conversation::Entity as Conversation};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_conversation(
        db: &DbConn,
        form_data: conversation::Model,
    ) -> Result<conversation::ActiveModel, DbErr> {
        conversation::ActiveModel {
            title: Set(form_data.title.to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_conversation_by_id(
        db: &DbConn,
        id: i32,
        form_data: conversation::Model,
    ) -> Result<conversation::Model, DbErr> {
        let post: conversation::ActiveModel = Conversation::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        conversation::ActiveModel {
            id: post.id,
            title: Set(form_data.title.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_conversation(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let conversation: conversation::ActiveModel = Conversation::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        conversation.delete(db).await
    }

    pub async fn delete_all_conversations(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Conversation::delete_many().exec(db).await
    }
}
