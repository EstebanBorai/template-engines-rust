use lazy_static;
use serde_derive;
use tera::Context;
use tera::Tera;

const ID_TEMPLATE: &str = "id_template";
const PERSON_ID_TEMPLATE: &str = "person_id_template";
const POSSIBLE_PERSON_TEMPLATE: &str = "possible_person_template";
const MULTIPLE_PERSON_ID_TEMPLATE: &str = "multiple_person_id_template";

lazy_static::lazy_static! {
    pub static ref TERA: Tera = Tera::new("src/templates/**").unwrap();
}

fn main() {
    let mut tera_engine = Tera::default();

    tera_engine
        .add_raw_template(ID_TEMPLATE, "Identifier: {{id}}.")
        .unwrap();

    let mut numeric_id = Context::new();

    numeric_id.insert("id", &7362);

    println!("{}", tera_engine.render(ID_TEMPLATE, &numeric_id).unwrap());

    // usage with structs
    tera_engine
        .add_raw_template(PERSON_ID_TEMPLATE, "Person id: {{person.id}}")
        .unwrap();

    #[derive(serde_derive::Serialize)]
    struct Person {
        id: i32,
        name: String,
    }

    let mut person_ctx = Context::new();

    person_ctx.insert(
        "person",
        &Person {
            id: 534,
            name: "Mary".to_string(),
        },
    );

    println!(
        "{}",
        tera_engine.render(PERSON_ID_TEMPLATE, &person_ctx).unwrap()
    );

    tera_engine
        .add_raw_template(
            POSSIBLE_PERSON_TEMPLATE,
            "{%if person%} Id: {{person.id}}\
        {%else%} No person \
        {%endif%}",
        )
        .unwrap();

    println!(
        "{}",
        tera_engine
            .render(POSSIBLE_PERSON_TEMPLATE, &person_ctx)
            .unwrap()
    );
    println!(
        "{}",
        tera_engine
            .render(POSSIBLE_PERSON_TEMPLATE, &numeric_id)
            .unwrap()
    );

    tera_engine
        .add_raw_template(
            MULTIPLE_PERSON_ID_TEMPLATE,
            "{%for p in persons%}\
                Id: {{p.id}};\n\
                {%endfor%}",
        )
        .unwrap();

    let mut persons_ctx = Context::new();

    persons_ctx.insert(
        "persons",
        &vec![
            Person {
                id: 123,
                name: "Esteban".to_string(),
            },
            Person {
                id: 456,
                name: "Sarasa".to_string(),
            },
        ],
    );

    println!(
        "{}",
        tera_engine
            .render(MULTIPLE_PERSON_ID_TEMPLATE, &persons_ctx)
            .unwrap()
    );

    println!("{}", TERA.render("templ_id.txt", &numeric_id).unwrap());
}
