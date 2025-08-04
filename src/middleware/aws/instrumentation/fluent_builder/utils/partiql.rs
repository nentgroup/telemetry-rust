use super::AsAttribute;
use crate::{KeyValue, semconv};

/// Represents a parsed table reference from a PartiQL statement
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct TableReference {
    pub name: String,
    pub index_name: Option<KeyValue>,
}

impl TableReference {
    pub fn new(table_part: impl Into<String>, index_part: impl AsAttribute) -> Self {
        Self {
            name: table_part.into(),
            index_name: index_part.as_attribute(semconv::AWS_DYNAMODB_INDEX_NAME),
        }
    }
}

impl From<&str> for TableReference {
    fn from(value: &str) -> Self {
        parse_partiql_statement(value).unwrap_or_default()
    }
}

const STATEMENTS: [(&str, Option<&str>); 4] = [
    ("SELECT", Some("FROM")),
    ("DELETE", Some("FROM")),
    ("INSERT", Some("INTO")),
    ("UPDATE", None),
];

/// Extracts table name and optional index name from PartiQL statements.
fn parse_partiql_statement(statement: &str) -> Option<TableReference> {
    let mut tokens = statement.split_whitespace();
    let first_token = tokens.next()?;

    let next_clause = STATEMENTS.into_iter().find_map(|(clause, next_clause)| {
        first_token
            .eq_ignore_ascii_case(clause)
            .then_some(next_clause)
    })?;

    if let Some(clause) = next_clause {
        loop {
            if tokens.next()?.eq_ignore_ascii_case(clause) {
                break;
            }
        }
    }

    let table_token = tokens.next()?;
    Some(parse_table_identifier(table_token))
}

/// Parses a table identifier that may include an index (e.g., "table"."index")
fn parse_table_identifier(id: &str) -> TableReference {
    let (table_part, index_part) = if id.starts_with('"') && id.ends_with('"') {
        let mut parts = id[1..id.len() - 1].split(r#"".""#);
        (parts.next().unwrap_or_default(), parts.next())
    } else {
        (id, None)
    };

    TableReference::new(table_part, index_part)
}

#[cfg(test)]
mod tests {
    use assert2::assert;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(r#"SELECT * FROM "Users" WHERE id = 1"#, "Users", None)]
    #[case("SELECT * FROM Users WHERE id = 1", "Users", None)]
    #[case("select * from Users where id = 1", "Users", None)]
    #[case(
        r#"SELECT * FROM "Users"."EmailIndex" WHERE email = 'test@example.com'"#,
        "Users",
        Some("EmailIndex")
    )]
    #[case(r#"INSERT INTO "Music" VALUE {'Artist': 'Acme Band', 'SongTitle': 'PartiQL Rocks'}"#, "Music", None)]
    #[case("INSERT INTO Orders VALUE {'id': 1, 'total': 100}", "Orders", None)]
    #[case("insert into Orders value {'id': 1, 'total': 100}", "Orders", None)]
    #[case(
        r#"
            INSERT
            INTO
            "Music"
            VALUE
            {'Artist': 'Acme Band', 'SongTitle': 'PartiQL Rocks'}
        "#,
        "Music",
        None
    )]
    #[case(
        r#"UPDATE "Music" SET AwardsWon=1 WHERE Artist='Acme Band'"#,
        "Music",
        None
    )]
    #[case("UPDATE Orders SET total = 150 WHERE id = 1", "Orders", None)]
    #[case(r#"DELETE FROM "Music" WHERE Artist='Acme Band'"#, "Music", None)]
    #[case("DELETE FROM Orders WHERE id = 1", "Orders", None)]
    #[case("delete from Orders where id = 1", "Orders", None)]
    #[case(
        r#"
            SELECT OrderID, Total
            FROM "Orders"."StatusIndex"
            WHERE OrderID = 1
        "#,
        "Orders",
        Some("StatusIndex")
    )]
    #[case(
        r#"
            SELECT OrderID, Total
            FROM "Orders"
            WHERE OrderID IN [1, 2, 3]
            ORDER BY OrderID DESC
        "#,
        "Orders",
        None
    )]
    #[case("INVALID STATEMENT", "", None)]
    fn test_parse_statement(
        #[case] statement: &str,
        #[case] expected_table_name: &str,
        #[case] expected_index_name: Option<&str>,
    ) {
        let table = TableReference::from(statement);
        assert!(table.name == expected_table_name);

        let index_name = table.index_name.map(|kv| kv.value.as_str().to_string());
        assert!(index_name.as_deref() == expected_index_name);
    }
}
