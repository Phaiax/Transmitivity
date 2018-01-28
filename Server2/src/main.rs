#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate reqwest;
extern crate url;
extern crate regex;
extern crate failure;
#[macro_use] extern crate lazy_static;
extern crate rand;

//use rocket_contrib::Json;
use rocket_contrib::Template;
use std::path::PathBuf;
use rocket::response::NamedFile;
use std::collections::HashMap;
use std::path::Path;
use rocket::response::content;
use std::sync::Mutex;
#[allow(unused_imports)]
use failure::{Error, Fail, ResultExt, err_msg};
use regex::bytes::Regex;
use std::sync::atomic::{AtomicUsize, Ordering};


#[get("/")]
fn index() -> Template {
    let context : HashMap<String, String> = HashMap::new();
    Template::render("index", &context)
}

#[get("/ip")]
fn ip() -> Template {
    let context : HashMap<String, String> = HashMap::new();
    Template::render("ip", &context)
}

#[get("/admin")]
fn admin() -> Template {
    let context : HashMap<String, String> = HashMap::new();
    Template::render("admin", &context)
}

#[get("/static/<file..>")]
fn static_(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}


#[get("/admin/next_game")]
fn next_game(game_state: rocket::State<GameState>) -> content::Json<String> {
    game_state.next_game();
    content::Json(json!({"result": "ok"}).to_string())
}


fn keyword_to_imageurl(keyword : &str) -> Result<String, Error> {
    lazy_static! {
        //static ref RE: Regex = Regex::new(r#"<img height="[0-9]+" src="(https://encrypted[^"]*)" [^>]*>"#)
        static ref RE: Regex = Regex::new(r#"<img height="[0-9]+" src="(https://encrypted[^"]*)""#)
                .unwrap();
    }

    let mut url = "https://www.google.de/search?tbm=isch&q=".to_owned();
    for enc in url::form_urlencoded::byte_serialize(keyword.as_bytes()) {
        url.push_str(enc);
    }
    url.push_str("&tbs=imgo:1&gws_rd=cr&dcr=0");
    let mut resp = reqwest::get(&url)?;
    if resp.status() != reqwest::StatusCode::Ok {
        return Err(err_msg("no internet?"));
    }
    use std::io::Read;
    let mut htmlsrc = vec![];
    resp.read_to_end(&mut htmlsrc)?;

    // println!("{}", String::from_utf8_lossy(&htmlsrc[..]));

    match RE.captures(&htmlsrc[..]) {
        Some(caps) => {
            Ok( String::from_utf8( caps.get(1).unwrap().as_bytes().to_owned() )? )
        },
        None => {
            Err(err_msg("no img found"))
        }

    }

}

#[get("/add_image/<keyword>")]
fn add_image(keyword : String, game_state: rocket::State<GameState>) -> content::Json<String> {
    println!("{:?}", keyword);

    match keyword_to_imageurl(&keyword) {
        Ok(imgageurl) => {
            match *game_state.game.lock().unwrap() {
                Game::GuessImage { ref mut urls, .. } => {
                    urls.push(imgageurl);
                    content::Json(json!({"result": "ok"}).to_string())
                }
            }
        },
        Err(e) => {
            println!("{:?}", e);
            content::Json(json!({"result": "err", "msg": format!("{:?}", e) }).to_string())
        }
    }
}



#[derive(Serialize)]
struct NewImages<'a, 'b> {
    urls : &'a [String],
    last_id : usize,
    ipquest: &'b str,
    round : usize,
}



#[get("/get_quest_and_images/<from_id>")]
#[allow(unreachable_patterns)]
fn get_quest_and_images(mut from_id : usize, game_state: rocket::State<GameState>)
    -> content::Json<String> {

    match *game_state.game.lock().unwrap() {
        Game::GuessImage { ref ipquest, ref urls, .. } => {
            from_id = if from_id < urls.len() { from_id } else { urls.len() };
            let json_str = serde_json::to_string(
                &NewImages{
                    urls: &urls[from_id..],
                    last_id: urls.len(),
                    ipquest: &ipquest,
                    round : game_state.round.load(Ordering::Relaxed),
                }).unwrap();
            content::Json(json_str)
        },
        _ => {
            content::Json(json!({}).to_string())
        }
    }
}

#[get("/xpquest")]
fn xpquest(game_state: rocket::State<GameState>) -> content::Json<String> {
    match *game_state.game.lock().unwrap() {
        Game::GuessImage { ref xpquest , .. } => {

            content::Json(json!({ "xpquest" : xpquest}).to_string())


        }
    }
}


#[derive(Debug)]
enum Game {
    /// ipquest: immersive player quest
    /// xpquest: external players quest
    GuessImage{ ipquest : String , xpquest: String, urls : Vec<String> }
}

#[derive(Debug)]
struct GameState {
    game : Mutex<Game>,
    round : AtomicUsize,
}

impl GameState {
    fn next_game(&self) {
        let saying = SAYINGS[rand::random::<usize>() % SAYINGS.len()];
        let mut game = self.game.lock().unwrap();
        *game = Game::GuessImage{
            ipquest : "Gesucht ist ein Sprichwort!".into(),
            xpquest : format!("Vermittle das Sprichwort: {}", saying),
            urls : vec![],
        };
        self.round.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let game = Game::GuessImage{
        ipquest:"Gesucht ist ein Sprichwort!".into(),
        xpquest : "Vermittle das Sprichwort: Der frühe Vogel fängt den Wurm!".into(),
        urls : vec![],
    };

    rocket::ignite()
        .mount("/", routes![index, add_image, get_quest_and_images, static_, ip, xpquest, admin, next_game])
        .manage(GameState {
            game : Mutex::new(game),
            round : AtomicUsize::new(0),
        })
        .attach(Template::fairing())
        .launch();
}


const SAYINGS: &[&str] = &[
    "Abwarten und Tee trinken.",
    "Adel verpflichtet.",
    "Alle Wege führen nach Rom.",
    "Aller Anfang ist schwer.",
    "Aller guten Dinge sind drei.",
    "Alles Gute kommt von oben.",
    "Alles neu macht der Mai.",
    "Alter geht vor Schönheit.",
    "Alter schützt vor Torheit nicht." ,
    "Auch ein blindes Huhn findet mal ein Korn.",
    "Auf alten Pferden lernt man reiten.",
    "Auge um Auge, Zahn um Zahn.",
    "Aus den Augen, aus dem Sinn.",
    "Aus Schaden wird man klug.",
    "Außen hui und innen pfui.",
    "Besser arm dran als Arm ab.",
    "Besser den Spatz in der Hand, als die Taube auf dem Dach.",
    "Besser spät als nie.",
    "Buchen sollst du suchen, Eichen sollst du weichen.",
    "Da beißt die Maus keinen Faden ab.",
    "Da liegt der Hase im Pfeffer.",
    "Da liegt der Hund begraben.",
    "Das fünfte Rad am Wagen sein.",
    "Das Leben ist kein Wunschkonzert",
    "Das Leben ist kein Zuckerschlecken.",
    "Das schlägt dem Fass den Boden aus.",
    "Den letzten beißen die Hunde!",
    "Den Nagel auf den Kopf treffen.",
    "Den Wald vor lauter Bäumen nicht sehen.",
    "Der Apfel fällt nicht weit vom Stamm.",
    "Der dümmste Bauer erntet die dicksten Kartoffeln.",
    "Der Esel nennt sich immer zuerst.",
    "Der Fisch stinkt vom Kopf her.",
    "Der frühe Vogel fängt den Wurm.",
    "Der Klügere gibt nach.",
    "Der Zweck heiligt die Mittel.",
    "Des einen Leid ist des anderen Freud'.",
    "Die Ratten verlassen das sinkende Schiff.",
    "Dummheit schützt vor Strafe nicht.",
    "Eile mit Weile.",
    "Ein jeder ist seines Glückes Schmied.",
    "Ein Unglück kommt selten allein.",
    "Früh übt sich, was ein Meister werden will.",
    "Geld regiert die Welt.",
    "Harte Schale und weicher Kern.",
    "In der Kürze liegt die Würze.",
    "Keine Antwort ist auch eine Antwort.",
    "Kleinvieh macht auch Mist.",
    "Kommt Zeit, kommt Rat.",
    "Lachen ist die beste Medizin.",
    "Liebe macht blind.",
    "Man soll das Eisen schmieden, solange es heiß ist.",
    "Nobel geht die Welt zu Grunde.",
    "Ordnung ist das halbe Leben.",
    "Probieren geht über studieren!",
    "Schlafende Hunde soll man nicht wecken.",
    "Sport ist Mord.",
    "Stille Wasser sind tief.",
    "Viel Köche verderben den Brei.",
    "Wer A sagt, muss auch B sagen.",
    "Wer nicht wagt, der nicht gewinnt.",
    "Wer nichts wird, wird Wirt.",
    "Wer schön sein will, muss leiden.",
    "Wer zuerst kommt, mahlt zuerst.",
    "Wo gehobelt wird, da fallen Späne.",
    "Ein Satz mit x: Das war wohl nix.",
];