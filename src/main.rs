use musicplayer_continue::{set_window, Music};
use rand::prelude::*;
use sfml::graphics::{RenderTarget, Transformable};
use std::process::exit;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use winapi::shared::windef::HWND;

fn main() {
    let env: Vec<String> = std::env::args().collect();
    for item in env {
        match item.as_str() {
            "-gen-list-json" => {
                std::fs::create_dir_all(std::path::Path::new("./resources/music")).unwrap();
                std::fs::write(
                    std::path::Path::new("./resources/list1.json"),
                    "{\n  \"version\": 100,\n  \"show_warning\": true,\n  \"mode\": \"random\",\n  \"font\": \"Minecraft AE.ttf\",\n  \"icon\": \"music_disc_13.png\",\n  \"background_info\": {\n    \"image\": \"toasts.png\",\n    \"rect\": [ 0, 0, 480, 96 ],\n    \"scale\": [ 3.0, 3.0 ]\n  },\n  \"background_warn\": {\n    \"image\": \"toasts.png\",\n    \"rect\": [ 0, 192, 280, 96 ],\n    \"scale\": [ 3.0, 3.0 ]\n  },\n  \"music\": []\n}\n",
                )
                .unwrap();

                println!("已成功的生成示例文件.");
                exit(0);
            }
            _ => (),
        }
    }

    let radio = rodio::OutputStream::try_default();

    match radio {
        Err(ref error) => {
            println!("你的音频设备似乎没有正确设置. 错误: {}\n请按回车键退出...", error);
        },
        _ => (),
    }

    let (_stream, stream_handle) = radio .unwrap();

    let list_json_path = std::path::Path::new("./resources/list.json");
    if !list_json_path.exists() {
        panic!("找不到 ./resources/list.json 文件");
    }

    let list_json_str = std::fs::read_to_string("./resources/list.json").unwrap();
    let list_json: serde_json::Value = serde_json::from_str(&list_json_str).unwrap();

    drop(list_json_str);

    let mut music = Vec::new();

    list_json["music"]
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|item| {
            let file = String::from(item["file"].as_str().unwrap());
            let name = String::from(item["name"].as_str().unwrap());

            println!("已添加 {} 从 ./resources/music/{}", name, file);

            music.push(Music::new(file, name));
        });

    let font = sfml::graphics::Font::from_file(&format!(
        "./resources/{}",
        list_json["font"].as_str().unwrap()
    ))
    .unwrap();

    let background_info_rect = list_json["background_info"]["rect"].as_array().unwrap();
    let background_info_rect = sfml::graphics::IntRect::new(
        background_info_rect[0].as_i64().unwrap() as i32,
        background_info_rect[1].as_i64().unwrap() as i32,
        background_info_rect[2].as_i64().unwrap() as i32,
        background_info_rect[3].as_i64().unwrap() as i32,
    );
    let mut background_info_texture = sfml::graphics::Texture::new().unwrap();
    background_info_texture
        .load_from_file(
            &format!(
                "./resources/{}",
                list_json["background_info"]["image"].as_str().unwrap()
            ),
            background_info_rect,
        )
        .unwrap();

    let mut background_info = sfml::graphics::Sprite::new();
    background_info.set_texture(&background_info_texture, true);

    let mut play_now = sfml::graphics::Text::new("Play Now", &font, 24);
    play_now.set_fill_color(sfml::graphics::Color::YELLOW);
    let mut music_name = sfml::graphics::Text::new("string", &font, 24);

    let mut popup = musicplayer_continue::Popup::new(
        1.0_f64,
        0.0_f64 - background_info.local_bounds().width as f64,
        0.0_f64,
        3.0_f64,
    );

    // 创建消息通道, tx 是生产者, rx 是消费者
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        loop {
            let random_num = rng.gen_range(0..music.len());
            let music = &music[random_num];

            println!("正在播放 {}...", music.name);
            tx.send(String::from(&music.name)).unwrap();

            let file = std::fs::File::open(format!("./resources/music/{}", music.file)).unwrap();
            let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
            sink.append(source);
            sink.play();

            sink.sleep_until_end();
        }
    });

    let mut window = sfml::graphics::RenderWindow::new(
        sfml::window::VideoMode::new(1280, 720, 24),
        "",
        sfml::window::Style::NONE,
        &sfml::window::ContextSettings::default(),
    );
    window.set_position(sfml::system::Vector2i::new(0, 0));
    set_window(window.system_handle() as HWND);

    loop {
        loop {
            let event = window.poll_event();

            match event {
                None => break,
                Some(event) => match event {
                    sfml::window::Event::Closed => window.close(),
                    _ => (),
                },
            }
        }

        if !popup.finished() && popup.started {
            background_info.set_position(sfml::system::Vector2f::new(popup.calc() as f32, 0.0));
            play_now.set_position(sfml::system::Vector2f::new(
                popup.calc() as f32 + 20.0_f32,
                15.0,
            ));
            music_name.set_position(sfml::system::Vector2f::new(
                popup.calc() as f32 + 20.0_f32,
                50.0,
            ));

            window.clear(sfml::graphics::Color::BLACK);
            window.draw(&background_info);
            window.draw(&play_now);
            window.draw(&music_name);
            window.display();
            set_window(window.system_handle() as HWND);
        } else {
            window.clear(sfml::graphics::Color::BLACK);
            window.display();
            window.set_active(false);

            let received = rx.recv().unwrap();
            println!("另一线程：已接受到消息 {}", received);
            music_name.set_string(&received);
            popup.reset();
        }
    }
}
