use ggez::{Context, GameResult, graphics::{self, Canvas, DrawParam, Text, Drawable}, event};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::input::keyboard::KeyInput;
use ggez::winit::event::VirtualKeyCode;
use rand::Rng;
use std::time::{Duration, Instant};

// Sabitler
const GRID_SIZE: i32 = 15; // Oyun alanının boyutunu belirler
const CELL_SIZE: f32 = 40.0; // Her bir hücrenin piksel boyutunu belirler
const SCREEN_SIZE: f32 = GRID_SIZE as f32 * CELL_SIZE; // Ekran boyutunu hesaplar
const GHOST_SPEED: f32 = 2.0; // Hayaletlerin hareket hızını belirler

// Oyun alanındaki hücre tipleri
#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Empty, // Boş hücre
    Wall, // Duvar hücresi
    Dot, // Nokta hücresi
}

// Hayalet yapısı
struct Ghost {
    pos: (f32, f32), // Hayaletin pozisyonu (x, y)
    direction: (f32, f32), // Hayaletin hareket yönü (x, y)
    last_move: Instant, // Hayaletin son hareket zamanı
}

impl Ghost {
    // Yeni bir hayalet oluşturur
    fn new(x: f32, y: f32) -> Self {
        Ghost {
            pos: (x, y),
            direction: (1.0, 0.0), // Başlangıçta sağa doğru hareket eder
            last_move: Instant::now(),
        }
    }

    // Hayaletin pozisyonunu günceller
    fn update(&mut self, walls: &Vec<Vec<TileType>>) {
        // Hayalet hareket hızı kontrolü
        if self.last_move.elapsed() < Duration::from_millis((1.0 / GHOST_SPEED * 1000.0) as u64) {
            return;
        }

        // Olası hareket yönleri
        let possible_directions = [
            (1.0, 0.0),
            (-1.0, 0.0),
            (0.0, 1.0),
            (0.0, -1.0),
        ];

        // Geçerli hareket yönlerini saklar
        let mut available_directions = Vec::new();
        for &dir in &possible_directions {
            let new_x = self.pos.0 + dir.0;
            let new_y = self.pos.1 + dir.1;
            
            // Yeni pozisyonun oyun alanı içinde ve duvar olmaması kontrolü
            if new_x >= 0.0 && new_x < GRID_SIZE as f32 && 
               new_y >= 0.0 && new_y < GRID_SIZE as f32 &&
               walls[new_y as usize][new_x as usize] != TileType::Wall {
                available_directions.push(dir);
            }
        }

        // Rastgele bir geçerli yön seç ve hayaleti hareket ettir
        if !available_directions.is_empty() {
            let idx = rand::thread_rng().gen_range(0..available_directions.len());
            self.direction = available_directions[idx];
            self.pos.0 += self.direction.0;
            self.pos.1 += self.direction.1;
        }

        // Son hareket zamanını güncelle
        self.last_move = Instant::now();
    }
}

// Oyunun ana durumu
struct MainState {
    player_pos: (i32, i32), // Oyuncunun pozisyonu (x, y)
    ghosts: Vec<Ghost>, // Hayaletlerin listesi
    walls: Vec<Vec<TileType>>, // Oyun alanındaki hücrelerin listesi
    score: i32, // Oyuncunun skoru
}

impl MainState {
    // Yeni bir oyun durumu oluşturur
    fn new() -> GameResult<MainState> {
        let mut walls = vec![vec![TileType::Empty; GRID_SIZE as usize]; GRID_SIZE as usize];
        
        // Labirent oluştur
        for x in 0..GRID_SIZE as usize {
            for y in 0..GRID_SIZE as usize {
                if (x % 3 == 0 || y % 3 == 0) && rand::thread_rng().gen_ratio(1, 3) {
                    walls[y][x] = TileType::Wall;
                } else if rand::thread_rng().gen_ratio(1, 2) {
                    walls[y][x] = TileType::Dot;
                }
            }
        }

        // Başlangıç noktasını temizle
        let center = GRID_SIZE as usize / 2;
        walls[center][center] = TileType::Empty;
        walls[center-1][center] = TileType::Empty;
        walls[center+1][center] = TileType::Empty;
        walls[center][center-1] = TileType::Empty;
        walls[center][center+1] = TileType::Empty;

        // Hayaletleri oluştur
        let mut ghosts = Vec::new();
        for _ in 0..4 {
            let mut ghost_x;
            let mut ghost_y;
            loop {
                ghost_x = rand::thread_rng().gen_range(0..GRID_SIZE) as f32;
                ghost_y = rand::thread_rng().gen_range(0..GRID_SIZE) as f32;
                if walls[ghost_y as usize][ghost_x as usize] == TileType::Empty {
                    break;
                }
            }
            ghosts.push(Ghost::new(ghost_x, ghost_y));
        }

        Ok(MainState {
            player_pos: (GRID_SIZE / 2, GRID_SIZE / 2),
            ghosts,
            walls,
            score: 0,
        })
    }

    // Verilen pozisyonun çarpışma olup olmadığını kontrol eder
    fn check_collision(&self, pos: (i32, i32)) -> bool {
        if pos.0 < 0 || pos.0 >= GRID_SIZE || pos.1 < 0 || pos.1 >= GRID_SIZE {
            return true;
        }
        self.walls[pos.1 as usize][pos.0 as usize] == TileType::Wall
    }
}

// Oyun olaylarını işler
impl event::EventHandler for MainState {
    // Oyun durumunu günceller
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Hayaletleri güncelle
        for ghost in &mut self.ghosts {
            ghost.update(&self.walls);
            
            // Hayalet ile çarpışma kontrolü
            let ghost_grid_pos = (ghost.pos.0.round() as i32, ghost.pos.1.round() as i32);
            if ghost_grid_pos == self.player_pos {
                // Oyunu yeniden başlat
                *self = MainState::new()?;
                return Ok(());
            }
        }

        // Oyuncu hareketini işle
        if let Some(key) = ctx.keyboard.pressed_keys().iter().next() {
            let (x, y) = self.player_pos;
            let new_pos = match key {
                VirtualKeyCode::Up => (x, y - 1),
                VirtualKeyCode::Down => (x, y + 1),
                VirtualKeyCode::Left => (x - 1, y),
                VirtualKeyCode::Right => (x + 1, y),
                _ => (x, y),
            };

            // Çarpışma yoksa oyuncuyu hareket ettir
            if !self.check_collision(new_pos) {
                self.player_pos = new_pos;
                // Nokta yeme kontrolü
                if self.walls[new_pos.1 as usize][new_pos.0 as usize] == TileType::Dot {
                    self.walls[new_pos.1 as usize][new_pos.0 as usize] = TileType::Empty;
                    self.score += 10;
                }
            }
        }
        Ok(())
    }

    // Oyunu çizer
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Duvarları ve noktaları çiz
        for y in 0..GRID_SIZE as usize {
            for x in 0..GRID_SIZE as usize {
                match self.walls[y][x] {
                    TileType::Wall => {
                        let wall = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            graphics::Rect::new(
                                x as f32 * CELL_SIZE,
                                y as f32 * CELL_SIZE,
                                CELL_SIZE,
                                CELL_SIZE,
                            ),
                            graphics::Color::BLUE,
                        )?;
                        canvas.draw(&wall, DrawParam::default());
                    },
                    TileType::Dot => {
                        // Noktaları büyüt
                        let dot = graphics::Mesh::new_circle(
                            ctx,
                            graphics::DrawMode::fill(),
                            [x as f32 * CELL_SIZE + CELL_SIZE/2.0, y as f32 * CELL_SIZE + CELL_SIZE/2.0],
                            5.0, // Nokta boyutu büyütüldü
                            0.1,
                            graphics::Color::WHITE,
                        )?;
                        canvas.draw(&dot, DrawParam::default());
                    },
                    _ => {}
                }
            }
        }

        // Hayaletleri çiz
        for ghost in &self.ghosts {
            let ghost_mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [ghost.pos.0 * CELL_SIZE + CELL_SIZE/2.0, 
                 ghost.pos.1 * CELL_SIZE + CELL_SIZE/2.0],
                CELL_SIZE/2.0,
                0.1,
                graphics::Color::RED,
            )?;
            canvas.draw(&ghost_mesh, DrawParam::default());
        }

        // Pacman'i çiz
        let player = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [self.player_pos.0 as f32 * CELL_SIZE + CELL_SIZE/2.0, 
             self.player_pos.1 as f32 * CELL_SIZE + CELL_SIZE/2.0],
            CELL_SIZE/2.0,
            0.1,
            graphics::Color::YELLOW,
        )?;
        canvas.draw(&player, DrawParam::default());

        // Skoru göster
        let score_text = Text::new(format!("Skor: {}", self.score));
        canvas.draw(
            &score_text,
            DrawParam::default().dest([10.0, 10.0]).color(graphics::Color::YELLOW),
        );

        canvas.finish(ctx)?;
        Ok(())
    }
}

// Oyunun ana fonksiyonu
fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("pacman", "cursor")
        .window_setup(WindowSetup::default().title("PACMAN CLONE Rust Project"))
        .window_mode(WindowMode::default().dimensions(SCREEN_SIZE, SCREEN_SIZE));
    
    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
