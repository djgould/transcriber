use ::entity::{conversation, conversation::Entity as Conversation};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_conversation_by_id(
        db: &DbConn,
        id: i32,
    ) -> Result<Option<conversation::Model>, DbErr> {
        Conversation::find_by_id(id).one(db).await
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_conversations_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<conversation::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Conversation::find()
            .order_by_desc(conversation::Column::CreatedAt)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
