use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u8,
    address: Address,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Employee {
    #[serde(flatten)]
    person: Person,
    job_title: String,
    employee_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_flattened_employee_serialization() {
        let employee = Employee {
            person: Person {
                name: "Alice".to_string(),
                age: 30,
                address: Address {
                    street: "123 Main St".to_string(),
                    city: "Wonderland".to_string(),
                    zip: "12345".to_string(),
                },
            },
            job_title: "Engineer".to_string(),
            employee_id: 42,
        };

        let json = serde_json::to_string_pretty(&employee).unwrap();

        let expected_json = r#"
        {
          "name": "Alice",
          "age": 30,
          "address": {
            "street": "123 Main St",
            "city": "Wonderland",
            "zip": "12345"
          },
          "job_title": "Engineer",
          "employee_id": 42
        }
        "#;

        let deserialized: Employee = serde_json::from_str(&json).unwrap();
        dbg!(&deserialized);
        let expected: Employee = serde_json::from_str(expected_json).unwrap();

        assert_eq!(deserialized, expected);
    }
}
