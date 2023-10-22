use std::sync::{Arc, Mutex};

use askama::Template; // bring trait in scope

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Form, Router,
};

use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use uuid::Uuid;

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateContact {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct SearchContact {
    q: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Clone)]
struct Contact {
    id: String,
    name: String,
    email: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "base.html")]
struct LayoutTemplate;

#[derive(Template)]
#[template(path = "contact_list.html")]
struct ContactListTemplate<'a> {
    contacts: &'a Vec<Contact>,
}


#[derive(Template)]
#[template(path = "contact_single_row.html")]
struct ContactSingleRowTemplate<'a> {
    contact: &'a Contact,
}


#[derive(Template)]
#[template(path = "contact_form.html")]
struct ContactFormTemplate {}

struct AppState {
    contacts: Mutex<Vec<Contact>>,
}

#[tokio::main]
async fn main() {
    let shared_app_state = Arc::new(AppState {
        contacts: Mutex::new(vec![
            Contact {
                id: Uuid::new_v4().into(),
                name: "John".to_string(),
                email: "john@email.com".to_string(),
            },
            Contact {
                id: Uuid::new_v4().into(),
                name: "Jane".to_string(),
                email: "jane@email.com".to_string(),
            },
        ]),
    });
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(index))
        .route("/contacts", get(get_contacts))
        .route("/contacts", post(create_contact))
        .route("/contacts/search", post(search_contacts))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_app_state);

    println!("Listening on: http://0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_contacts(State(state): State<Arc<AppState>>) -> Html<String> {
    generate_contact_table_list(&state.contacts.lock().unwrap())
}

// basic handler that responds with a static string
async fn index() -> Html<String> {
    let index = Html(IndexTemplate.render().unwrap());
    index
}


async fn search_contacts(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<SearchContact>,
) -> Html<String> {
    // insert your application logic here
    let searched_contacts = state.contacts.lock().unwrap()
        .clone() // Not efficient, but it works for now
        .into_iter()
        .filter(|contact| contact.name.contains(&payload.q) || contact.email.contains(&payload.q))
        .collect();

    generate_contact_table_list(&searched_contacts)
}

async fn create_contact(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CreateContact>,
) -> Html<String> {
    // insert your application logic here
    let contact = Contact {
        id: Uuid::new_v4().into(),
        name: payload.name,
        email: payload.email,
    };

    state.contacts.lock().unwrap().push(contact.clone());

    generate_contact_single_row(&contact)
}

fn generate_contact_table_list(contacts: &Vec<Contact>) -> Html<String> {
    let contact_list_template = ContactListTemplate {
        contacts: &contacts,
    };
    let contact_list_html = Html(contact_list_template.render().unwrap());
    contact_list_html
}

fn generate_contact_single_row(contact: &Contact) -> Html<String> {
    let contact_single_row_template = ContactSingleRowTemplate {
        contact: &contact,
    };
    let contact_single_row_html = Html(contact_single_row_template.render().unwrap());
    contact_single_row_html
}
