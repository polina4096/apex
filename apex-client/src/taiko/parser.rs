use std::{collections::HashMap, path::PathBuf};

use intbits::Bits;
use wcore::time::Time;

use super::taiko_circle::{TaikoCircle, TaikoColor};

pub struct TimingPoint {
    pub time : Time,
    pub bpm  : f64,
}

pub struct VelocityPoint {
    pub time     : Time,
    pub velocity : f64,
}

pub struct Beatmap {
    pub objects  : Vec<TaikoCircle>,
    pub timing   : Vec<TimingPoint>,
    pub velocity : Vec<VelocityPoint>,

    pub velocity_multiplier : f32,

    pub audio: PathBuf,
}

type ParseError = ();

#[allow(clippy::bind_instead_of_map, clippy::match_like_matches_macro)]
#[allow(unused_variables, unused_assignments, clippy::result_unit_err)]
pub fn try_parse(data: &str) -> Result<Beatmap, ParseError> {
    let mut version_string = None;
    let mut objects_taiko = Vec::<TaikoCircle>::new();
    let mut timing_points = Vec::<TimingPoint>::new();
    let mut velocity_points = Vec::<VelocityPoint>::new();

    
    let mut table = HashMap::<&str, HashMap<&str, &str>>::new();
    let mut category: Option<&str> = None;
    for line in data.lines() {
        if let Some(char) = line.chars().nth(0)
        && char == '[' {
            category = Some(line);
            continue;
        }

        match category {
            Some("[TimingPoints]") => {
                if line.trim().is_empty() { continue }
                
                let mut parts = line.split(',');
                let Some(time_ms)     = parts.next().and_then(|x| x.parse::<i32>().ok()) else { continue };
                let Some(beat_length) = parts.next().and_then(|x| x.parse::<f64>().ok()) else { continue };
                let Some(uninherited) = parts.nth(4).and_then(|x| Some(x == "1"))        else { continue };
                
                if uninherited {
                    let bpm = (60.0 * 1000.0) / beat_length;
                    timing_points.push(TimingPoint {
                        time : Time::from_ms(time_ms),
                        bpm  : bpm,
                    });
                } else {
                    let velocity = -100.0 / beat_length;
                    velocity_points.push(VelocityPoint {
                        time     : Time::from_ms(time_ms),
                        velocity : velocity,
                    });
                }
            }

            Some("[HitObjects]") => {
                let mut parts = line.split(',');

                match table["[General]"]["Mode"] {
                    // Taiko
                    "1" => {
                        let Some(time_in_ms)  = parts.nth(2).and_then(|x| x.parse::<f64>().ok()) else { continue };
                        let Some(object_type) = parts.nth(1).and_then(|x| x.parse::<u8> ().ok()) else { continue };
                        
                        objects_taiko.push(
                            TaikoCircle {
                                time  : Time::from_ms(time_in_ms),
                                color : if object_type.bit(1) || object_type.bit(3) { TaikoColor::KAT } else { TaikoColor::DON },
                                big   : object_type.bit(2),
                            }
                        );
                    }

                    // Mania
                    "3" => {
                        let Ok(key_count) = table["[Difficulty]"]["CircleSize"].parse::<f64>() else { continue };

                        // x,y,time,type,hitSound,...
                        let Some(x_position)  = parts.next().and_then(|x| x.parse::<isize>().ok()) else { continue };
                        let Some(time_ms)     = parts.nth(1).and_then(|x| x.parse::<f64>  ().ok()) else { continue };
                        let Some(object_type) = parts.nth(1).and_then(|x| x.parse::<u8>   ().ok()) else { continue };

                        // objects_mania.push(
                        //     VsrgNote {
                        //         time     : Time::from_ms(time_ms),
                        //         key      : (x_position as f64 * key_count / 512.0).floor() as usize,
                        //         duration : if (0b10000000 & object_type) != 0 { // is a hold note
                        //             // ...,endTime:hitSample
                        //             let Some(end_time_ms) = parts.nth(1).and_then(|x| {
                        //                 let mut parts = x.split(':');
                        //                 parts.next().and_then(|x| x.parse::<f64>().ok())
                        //             }) else { continue };
                                    
                        //             Time::from_seconds(end_time_ms - time_ms)
                        //         } else { Time::zero() },
                        //     }
                        // );
                    }

                    _ => {}
                }
            }

            Some(category) => {
                if line.trim().is_empty() { continue }

                let mut parts = line.split(':');
                let Some(key)   = parts.next() else { continue };
                let Some(value) = parts.next() else { continue };

                table
                    .entry(category)
                    .or_default()
                    .insert(key.trim(), value.trim());
            }

            None => {
                if line.starts_with("osu file format v") {
                    version_string = Some(line.to_owned());
                }
            }
        }
    }

    if let Some(p) = timing_points   . get(0) && p.time != Time::zero() { timing_points   . insert(0, TimingPoint   { time: Time::zero(), bpm      : 60.0 }); }
    if let Some(p) = velocity_points . get(0) && p.time != Time::zero() { velocity_points . insert(0, VelocityPoint { time: Time::zero(), velocity :  1.0 }); }
    if timing_points   . is_empty() { timing_points   . insert(0, TimingPoint   { time: Time::zero(), bpm      : 60.0 }); }
    if velocity_points . is_empty() { velocity_points . insert(0, VelocityPoint { time: Time::zero(), velocity :  1.0 }); }
    
    objects_taiko.sort_by(|a, b| a.time.to_seconds().total_cmp(&b.time.to_seconds()));

    return match table["[General]"]["Mode"] {
        // Taiko
        "1" => Ok(Beatmap {
            objects  : objects_taiko,
            timing   : timing_points,
            velocity : velocity_points,

            velocity_multiplier : table["[Difficulty]"]["SliderMultiplier"].parse().unwrap(),
            
            audio : PathBuf::from(table["[General]"]["AudioFilename"]),
        }),

        // Mania
        // "3" => Ok(OsuBeatmap::ManiaBeatmap(Beatmap::<VsrgNote> {
        //     objects : objects_mania,
        //     timing  : timing_points,
        // })),

        _ => Err(())
    }
}