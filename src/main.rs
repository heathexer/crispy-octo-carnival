#[macro_use]
extern crate actix_web;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io;

#[derive(Serialize, Deserialize)]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    instructions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct RecipeList {
    recipes: Vec<Recipe>,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_recipe_names)
            .service(get_recipe)
            .service(add_recipe)
            .service(update_recipe)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

#[get("/recipes")]
async fn get_recipe_names(_req: HttpRequest) -> HttpResponse {
    // Open file and read it as a RecipeList struct
    let file = File::open("data.json").expect("Failed to open file");
    let reader = io::BufReader::new(file);
    let data: RecipeList = serde_json::from_reader(reader).unwrap();

    // Iterate through the recipe list and pull the name from each recipe, adding it to a vector
    let mut names = Vec::<Value>::new();
    for recipe in data.recipes {
        names.push(Value::String(recipe.name));
    }
    // Put vector with the key "recipeNames" into a json object
    let res = json!({ "recipeNames": Value::Array(names) });

    // Return a response with the list of recipe names and an OK status
    HttpResponse::Ok()
        .content_type("application/json")
        .json(res)
}

#[get("/recipes/details/{name}")]
async fn get_recipe(req: HttpRequest) -> HttpResponse {
    // Open file and read it as a RecipeList struct
    let file = File::open("data.json").expect("Failed to open file");
    let reader = io::BufReader::new(file);
    let data: RecipeList = serde_json::from_reader(reader).unwrap();

    // Iterate through recipe list and set res to recipe, leaving it as None if it doesn't exist
    let mut res: Option<Recipe> = None;
    for recipe in data.recipes {
        if recipe.name == &req.match_info()["name"] {
            res = Some(recipe);
            break;
        }
    }

    match res {
        // Return recipe details if found
        Some(recipe) => HttpResponse::Ok()
            .content_type("application/json")
            .json(json!({
                "details": {
                    "ingredients": serde_json::to_value(recipe.ingredients).unwrap(),
                    "numSteps": serde_json::to_value(recipe.instructions.len()).unwrap(),
                }
            })),
        // Return error if not found
        None => HttpResponse::NotFound()
            .content_type("application/json")
            .json({}),
    }
}

#[post("/recipes")]
async fn add_recipe(req: web::Json<Recipe>) -> HttpResponse {
    // Open file and read it as a RecipeList struct
    let file = File::open("data.json").expect("Failed to open file");
    let reader = io::BufReader::new(&file);
    let mut data: RecipeList = serde_json::from_reader(reader).unwrap();

    // Iterate through recipes to check if this recipe exists already
    for recipe in &data.recipes {
        if recipe.name == req.name {
            // If recipe exists, return error
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .json(json!({
                    "error": "Recipe already exists"
                }));
        }
    }

    // If recipe doesn't exist, add it to the RecipeList
    data.recipes.push(req.into_inner());
    // Then write the recipe list to the file
    let file_out = File::create("data.json").expect("Failed to create file");
    serde_json::to_writer_pretty(file_out, &data).expect("Failed to write to file");

    // Return confirmation response
    HttpResponse::Created().finish()
}

#[put("/recipes")]
async fn update_recipe(req: web::Json<Recipe>) -> HttpResponse {
    // Open file and read it as a RecipeList struct
    let file = File::open("data.json").expect("Failed to open file");
    let reader = io::BufReader::new(&file);
    let mut data: RecipeList = serde_json::from_reader(reader).unwrap();

    // Iterate through recipes to see if this one exists
    for i in 0..data.recipes.len() {
        if data.recipes[i].name == req.name {
            // If it does, update its contents with the contents of the request and write to file
            data.recipes[i] = req.into_inner();
            let file_out = File::create("data.json").expect("Failed to create file");
            serde_json::to_writer_pretty(file_out, &data).expect("Failed to write to file");

            // Return confirmation
            return HttpResponse::Created().finish();
        }
    }

    // If this recipe doesn't exist, return error that says so
    HttpResponse::NotFound()
        .content_type("application/json")
        .json(json!({
            "error": "Recipe does not exist"
        }))
}
