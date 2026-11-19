use crate::api::error::{ApiError, ApiResult};
use crate::model::enums::ResourceType;
use diesel::sql_types::{BigInt, Bool, Nullable};
use diesel::{PgConnection, QueryableByName, RunQueryDsl, sql_query};

/// Serialize graph edits until the transaction commits, including changes to automatic parents.
pub fn lock(conn: &mut PgConnection) -> ApiResult<()> {
    sql_query("SELECT pg_advisory_xact_lock(893791022)").execute(conn)?;
    Ok(())
}

/// An absent row means automatic; a row with no parent means explicitly unindented.
pub fn set(conn: &mut PgConnection, tag_id: i64, parent: Option<Option<i64>>) -> ApiResult<()> {
    if let Some(parent_id) = parent {
        if parent_id == Some(tag_id) {
            return Err(ApiError::CyclicDependency(ResourceType::Tag));
        }
        sql_query(
            "INSERT INTO tag_parent (tag_id, parent_id) VALUES ($1, $2)
             ON CONFLICT (tag_id) DO UPDATE SET parent_id = excluded.parent_id",
        )
        .bind::<BigInt, _>(tag_id)
        .bind::<Nullable<BigInt>, _>(parent_id)
        .execute(conn)?;
    } else {
        sql_query("DELETE FROM tag_parent WHERE tag_id = $1")
            .bind::<BigInt, _>(tag_id)
            .execute(conn)?;
    }
    Ok(())
}

#[derive(QueryableByName)]
struct CycleCheck {
    #[diesel(sql_type = Bool)]
    cyclic: bool,
}

pub fn verify_acyclic(conn: &mut PgConnection) -> ApiResult<()> {
    let result = sql_query(
        "WITH RECURSIVE
         single_implication AS (
             SELECT parent_id AS tag_id, min(child_id) AS parent_id
             FROM tag_implication GROUP BY parent_id HAVING count(*) = 1
         ),
         edge AS (
             SELECT child.id, parent.id AS parent_id
             FROM tag child
             LEFT JOIN tag_parent choice ON choice.tag_id = child.id
             LEFT JOIN single_implication automatic ON automatic.tag_id = child.id
             JOIN tag parent ON parent.id = CASE WHEN choice.tag_id IS NULL
                 THEN automatic.parent_id ELSE choice.parent_id END
             WHERE child.category_id = parent.category_id
         ),
         chain(origin, node) AS (
             SELECT tag_id, parent_id FROM tag_parent WHERE parent_id IS NOT NULL
             UNION
             SELECT chain.origin, edge.parent_id FROM chain JOIN edge ON edge.id = chain.node
         )
         SELECT EXISTS (SELECT 1 FROM chain WHERE origin = node) AS cyclic",
    )
    .get_result::<CycleCheck>(conn)?;
    if result.cyclic {
        return Err(ApiError::CyclicDependency(ResourceType::Tag));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{lock, set, verify_acyclic};
    use crate::api::error::{ApiError, ApiResult};
    use crate::model::tag::{NewTag, NewTagName};
    use crate::resource::field::Mask;
    use crate::resource::tag::{Field, TagInfo};
    use crate::schema::{tag, tag_category, tag_implication, tag_name};
    use diesel::{Connection, ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
    use serde_json::{json, to_value};
    use serial_test::serial;

    #[test]
    #[serial]
    fn verifies_parent_modes_and_mixed_cycles() -> ApiResult<()> {
        let mut conn = crate::test::get_connection()?;
        conn.test_transaction::<_, ApiError, _>(|conn| {
            lock(conn)?;
            let category_id = tag_category::table.select(tag_category::id).first(conn)?;
            let make_tag = |conn: &mut diesel::PgConnection| {
                NewTag { category_id, description: "" }
                    .insert_into(tag::table)
                    .returning(tag::id)
                    .get_result::<i64>(conn)
            };
            let child = make_tag(conn)?;
            let parent = make_tag(conn)?;
            NewTagName { tag_id: parent, order: 0, name: "display_parent_test" }
                .insert_into(tag_name::table)
                .execute(conn)?;
            let read = |conn: &mut diesel::PgConnection, id| -> ApiResult<_> {
                Ok(to_value(TagInfo::new_from_id(conn, id, Mask::from([Field::ParentTag]))?)?)
            };
            assert_eq!(read(conn, child)?, json!({"parentTag": {"mode": "auto"}}));
            set(conn, child, Some(Some(parent)))?;
            assert_eq!(read(conn, child)?, json!({"parentTag": {"mode": "tag", "name": "display_parent_test"}}));
            verify_acyclic(conn)?;
            diesel::insert_into(tag_implication::table)
                .values((tag_implication::parent_id.eq(parent), tag_implication::child_id.eq(child)))
                .execute(conn)?;
            assert!(matches!(verify_acyclic(conn), Err(ApiError::CyclicDependency(_))));
            set(conn, parent, Some(None))?;
            assert_eq!(read(conn, parent)?, json!({"parentTag": {"mode": "none"}}));
            verify_acyclic(conn)?;
            assert!(matches!(set(conn, child, Some(Some(child))), Err(ApiError::CyclicDependency(_))));
            Ok(())
        });
        Ok(())
    }
}
