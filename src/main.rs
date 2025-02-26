extern crate sdl2; 
extern crate serde;
extern crate serde_json;
// #[macro_use] extern crate json_derive;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS, Chunk, Music};

use rand::Rng;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::fs::File;
use std::io::Read;

const WIDTH: u32 = 20;
const HEIGHT: u32 = 20;
const PX: u32 = 30;
const PADDING: u32 = 20;

const BG:     Color = Color::RGB(20, 20, 20);
const GRAY:   Color = Color::RGB(30, 30, 30);
const WHITE:  Color = Color::RGB(240, 240, 240);
const BLACK:  Color = Color::RGB(30, 30, 30);
const GREEN1: Color = Color::RGB(15, 210, 15);
const GREEN2: Color = Color::RGB(10, 170, 10);
const GREEN3: Color = Color::RGB(0, 120, 0);
const RED:    Color = Color::RGB(240, 10, 20);
const TEXT:   Color = Color::RGB(230, 240, 250);

#[derive(PartialEq, Copy, Clone)]
enum Dir {
    Up,
    Down,
    Left,
    Right
}

struct Game {
    snake: Vec<[u32; 2]>,
    apples: Vec<[u32; 2]>,

    config: Config,

    cycle: u16,

    next_direction: Dir,
    direction: Dir
}

fn init_game(config: Config) -> Game {
    let mut game = Game{
        snake:  Vec::<[u32; 2]>::new(),
        apples: Vec::<[u32; 2]>::new(),

        config,

        cycle: 500,

        next_direction: Dir::Right,
        direction: Dir::Right,
    };

    game.snake.push([WIDTH/2-2, HEIGHT/2-1]);
    game.snake.push([WIDTH/2-1, HEIGHT/2-1]);
    game.snake.push([WIDTH/2, HEIGHT/2-1]);

    game.spawn_apples();

    game
}

impl Game {
    // fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>,
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
            ttf_context: &sdl2::ttf::Sdl2TtfContext) -> Result<(), Box<dyn std::error::Error>> {

        canvas.set_draw_color(GRAY);
        for y in 0..HEIGHT * 2 {
            for x in 0..WIDTH * 2 {
                if (x + y) % 2 == 0 {
                    let _ = canvas.fill_rect(Rect::new(
                        (PX*x/2+PADDING).try_into().unwrap(),
                        (PX*y/2+PADDING).try_into().unwrap(),
                        PX/2, PX/2));
                }
            }
        }

        let mut col = self.snake.len() % 2 == 0;
        for pos in &self.snake {
            if col {
                canvas.set_draw_color(GREEN1);
            } else {
                canvas.set_draw_color(GREEN2)
            }

            col = !col;

            let _ = canvas.fill_rect(Rect::new(
                <u32 as TryInto<i32>>::try_into(pos[0]*PX+PADDING).unwrap(),
                <u32 as TryInto<i32>>::try_into(pos[1]*PX+PADDING).unwrap(),
                PX, PX));
        }

        for pos in &self.apples {
            let _ = sdl2::gfx::primitives::DrawRenderer::thick_line(canvas,
                (pos[0]*PX+PX/2+PADDING).try_into().unwrap(),
                (pos[1]*PX+PADDING).try_into().unwrap(),
                (pos[0]*PX+PX/4*3+PADDING).try_into().unwrap(),
                (pos[1]*PX+PADDING).saturating_sub(PX/2).try_into().unwrap(),

                6, GREEN3);

            canvas.set_draw_color(RED);
            let _ = canvas.fill_rect(Rect::new(
                <u32 as TryInto<i32>>::try_into(pos[0]*PX+PADDING).unwrap(),
                <u32 as TryInto<i32>>::try_into(pos[1]*PX+PADDING).unwrap(),
                PX, PX));
        }

        /*
        match self.direction {
            Dir::Right => { pos[0] += PX/2 },
            Dir::Down  => { pos[1] += PX/2; pos[0] += PX },
            Dir::Left  => { pos[1] += PX; pos[0] += PX/2 },
            Dir::Up    => { pos[1] += PX/2 },
        }

        match self.direction {
            Dir::Right => { pos[1] += PX },
            Dir::Down  => { pos[0] -= PX },
            Dir::Left  => { pos[1] -= PX },
            Dir::Up    => { pos[0] += PX },
        }
        */

        let mut pos = self.snake[self.snake.len()-1];
        pos[0] *= PX;
        pos[1] *= PX;
        pos[0] += PX / 4;
        pos[1] += PX / 3;
        let mut pos2 = pos;
        pos2[0] += PX - PX / 2;

        pos[0]  += PADDING;
        pos[1]  += PADDING;
        pos2[0] += PADDING;
        pos2[1] += PADDING;

        let _ = sdl2::gfx::primitives::DrawRenderer::filled_circle(canvas,
            pos[0].try_into().unwrap(), pos[1].try_into().unwrap(), 5, WHITE);
        let _ = sdl2::gfx::primitives::DrawRenderer::filled_circle(canvas,
            pos2[0].try_into().unwrap(), pos2[1].try_into().unwrap(), 5, WHITE);
        let _ = sdl2::gfx::primitives::DrawRenderer::filled_circle(canvas,
            pos[0].try_into().unwrap(), pos[1].try_into().unwrap(), 3, BLACK);
        let _ = sdl2::gfx::primitives::DrawRenderer::filled_circle(canvas,
            pos2[0].try_into().unwrap(), pos2[1].try_into().unwrap(), 3, BLACK);

        let texture_creator = (*canvas).texture_creator();

        // Load a font
        let mut font = ttf_context.load_font("SourceCodePro-Regular.otf", 128)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(format!("Poäng: {}", self.snake.len() - 3).as_str())
            .blended(TEXT)
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let TextureQuery { width, height, .. } = texture.query();

        let _ = canvas.copy(&texture, None, Rect::new(8, 3, width/9, height/8))?;

        Ok(())
    }

    fn spawn_apples(&mut self) {
        'iter: loop {
            let new = [rand::rng().random_range(0..WIDTH-1), rand::rng().random_range(0..HEIGHT-1)];

            for pos in &self.snake {
                if new == *pos {
                    continue 'iter
                }
            }

            for pos in &mut self.apples {
                if new == *pos {
                    continue 'iter
                }
            }

            self.apples.push(new);
            break
        }
    }

    fn update(&mut self, nom: &Chunk) -> bool {
        let mut last = self.snake[self.snake.len()-1];

        self.direction = self.next_direction;

        match self.direction {
            Dir::Up    => {
                if last[1] == 0 {
                    if self.config.walls_kill {
                        return false
                    } else {
                        last[1] = HEIGHT - 1
                    }

                } else {
                    last[1] -= 1
                }
            },
            Dir::Left  => {
                if last[0] == 0 {
                    if self.config.walls_kill {
                        return false
                    } else {
                        last[0] = HEIGHT - 1
                    }

                } else {
                    last[0] -= 1
                }
            },
            Dir::Down  => {last[1] += 1},
            Dir::Right => {last[0] += 1}
        }

        if last[0] >= WIDTH {
            if self.config.walls_kill {
                return false
            } else {
                last[0] = 0
            }
        }

        if last[1] >= HEIGHT {
            if self.config.walls_kill {
                return false
            } else {
                last[1] = 0
            }
        }

        self.snake.push(last);

        let mut found = false;

        for i in 0..self.apples.len() {
            if last == self.apples[i] {
                self.apples.remove(i);
                self.spawn_apples();
                found = true;
                break
            }
        }

        if !found {
            self.snake.remove(0);

        } else {
            if self.config.audio {
                let _ = sdl2::mixer::Channel::all().play(&nom, 0);
            }

            let new = match self.snake.len() - 3 {
                3  => 400,
                10 => 300,
                20 => 200,
                30 => 100,
                _  => 0
            };

            if new != 0 {
                self.cycle = new;
                self.spawn_apples();
                println!("Level up! Poäng: {}", self.snake.len() - 3);
            }
        }

        for i in 0..self.snake.len()-1 {
            if last == self.snake[i] {
                return false
            }
        }

        return true
    }

    fn change_dir(&mut self, dir: Dir) {
        match dir {
            Dir::Up    => { if self.direction != Dir::Down  { self.next_direction = dir }},
            Dir::Down  => { if self.direction != Dir::Up    { self.next_direction = dir }},
            Dir::Left  => { if self.direction != Dir::Right { self.next_direction = dir }},
            Dir::Right => { if self.direction != Dir::Left  { self.next_direction = dir }},
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    music: bool,
    audio: bool,
    walls_kill: bool,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*******
     * SDL *
     *******/

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Episkt maskspel.",
        WIDTH*PX+PADDING*2, HEIGHT*PX+PADDING*2)
        .position_centered()
        .build()
        .unwrap();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
 
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let sdl = sdl2::init()?;

    /********
     * Ljud *
     ********/

    let _audio = sdl.audio()?;

    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    let _mixer_context =
        sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG)?;

    // Number of mixing channels available for sound effect `Chunk`s to play
    // simultaneously.
    sdl2::mixer::allocate_channels(4);

    let music = sdl2::mixer::Music::from_file(Path::new("doctor_worm.mp3"))?;
    Music::<'_>::set_volume(32);
    let nom = sdl2::mixer::Chunk::from_file(Path::new("crunch.mp3"))?;

    /**********
     * Config *
     **********/

    let mut file = File::open("config.json").expect("config.json not found!");
    let mut buff = String::new();
    file.read_to_string(&mut buff).unwrap();

    let config: Config = serde_json::from_str(&buff).unwrap();

    /*************
     * Game loop *
     *************/

    if config.music && config.audio {
        let _ = music.play(-1);
    }

    let mut game = init_game(config);
    let mut last = 0;
    println!("Mask för fan.");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running Ok(())
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {game.change_dir(Dir::Up)},
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {game.change_dir(Dir::Left)},
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {game.change_dir(Dir::Down)},
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {game.change_dir(Dir::Right)},
                _ => {}
            }
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Error!").as_millis();

        if now - last > game.cycle.into() {
            last = now;

            canvas.set_draw_color(BG);
            canvas.clear();

            if !game.update(&nom) {
                println!("Du dog för fan!\nPoäng: {} (sämst)", game.snake.len()-3);
                break 'running Ok(())
            }

            let _ = game.draw(&mut canvas, &ttf_context);

            canvas.present();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
}
