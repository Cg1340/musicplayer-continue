use easing::Exp;
use winapi::{
    shared::windef::HWND,
    um::winuser::{
        GetWindowLongA, SetLayeredWindowAttributes, SetWindowLongA, SetWindowPos, GWL_EXSTYLE,
        HWND_TOPMOST, LWA_COLORKEY, SWP_NOMOVE, SWP_NOSIZE, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    },
};

pub struct Music {
    pub file: String,
    pub name: String,
}

impl Music {
    pub fn new(file: String, name: String) -> Music {
        Music { file, name }
    }
}

pub struct Popup {
    pub start: f64,
    pub end: f64,
    pub duration: f64,
    pub ease: easing::Easing,
    pub time: std::time::Instant,
    pub residence_time: f64,
    pub retracting: bool,
}

impl Popup {
    pub fn new(duration: f64, start: f64, end: f64, residence_time: f64) -> Popup {
        Popup {
            start,
            end,
            duration,
            ease: easing::Easing {
                t: 0.0_f64,
                b: start,
                c: end - start,
                d: duration,
            },
            time: std::time::Instant::now(),
            residence_time,
            retracting: false,
        }
    }

    pub fn calc(&mut self) -> f64 {
        if self.time.elapsed().as_secs_f64() <= self.duration {
            self.ease.t = self.time.elapsed().as_secs_f64();
            return self.ease.calc_in();
        } else if self.time.elapsed().as_secs_f64() <= self.duration + self.residence_time {
            return self.end;
        } else {
            if !self.retracting {
                self.retracting = true;
                self.ease =
                    easing::Easing::new(0.0_f64, self.end, self.start - self.end, self.duration);
            }
            self.ease.t = self.time.elapsed().as_secs_f64() - self.residence_time - self.duration;
            return self.ease.calc_in();
        }
    }

    pub fn reset(&mut self) {
        self.retracting = false;
        self.time = std::time::Instant::now();
        self.ease = easing::Easing::new(0.0_f64, self.start, self.end - self.start, self.duration);
    }

    pub fn finished(&self) -> bool {
        self.time.elapsed().as_secs_f64() > self.duration * 2.0_f64 + self.residence_time
    }
}

pub mod easing {

    pub struct Easing {
        pub t: f64, // 表示动画开始以来经过的时间
        pub b: f64, // 动画的起点
        pub c: f64, // 从起点到终点的差值
        pub d: f64, // 完成动画所需的时间
    }

    pub trait Exp {
        fn calc_in(&self) -> f64;
    }

    impl Easing {
        pub fn new(t: f64, b: f64, c: f64, d: f64) -> Easing {
            Easing { t, b, c, d }
        }
    }

    impl Exp for Easing {
        fn calc_in(&self) -> f64 {
            if self.t == 0.0_f64 {
                return self.b;
            } else if self.t >= self.d {
                return self.b + self.c;
            } else {
                return self.c * (2.0_f64).powf(10.0_f64 * (self.t / self.d - 1.0_f64)) + self.b;
            }
        }
    }
}

pub fn set_window(hwnd: HWND) {
    unsafe {
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        let mut style = GetWindowLongA(hwnd, GWL_EXSTYLE);
        style = style | WS_EX_LAYERED as i32 | WS_EX_TOOLWINDOW as i32;
        SetWindowLongA(hwnd, GWL_EXSTYLE, style);
        SetLayeredWindowAttributes(hwnd, 0x000000, 0, LWA_COLORKEY);
    }
}
