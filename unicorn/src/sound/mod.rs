pub mod sound {
    use std::sync::mpsc;
    use unicorn::packet;
    use unicorn::UnicornCartridge;

    use chiptune::chiptune;

    use std::sync::{Arc, Mutex};

    pub struct SoundInternal {
        pub player: chiptune::Chiptune,
        pub csend: mpsc::Sender<Vec<u8>>,
        pub crecv: mpsc::Receiver<Vec<u8>>,
    }

    impl SoundInternal {
        pub fn new() -> SoundInternal {
            let (csend, crecv) = mpsc::channel();

            SoundInternal {
                player: chiptune::Chiptune::new(),
                csend: csend,
                crecv: crecv,
            }
        }

        pub fn init(&mut self) {}

        pub fn pause(&mut self) {
            info!("[SOUND] Pause");
            self.player.pause(1);
        }

        pub fn resume(&mut self) {
            info!("[SOUND] Resume");
            self.player.pause(0);
        }

        pub fn stop(&mut self) {
            info!("[SOUND] Stop");
            self.player.stop();
        }

        pub fn stop_chan(&mut self, chan: i32) {
            self.player.stop_chan(chan);
        }

        pub fn new_music(&mut self, cartridge: &mut UnicornCartridge, filename: String) -> i32 {
            // if filename != "" {
            // if !cartridge.music_tracks.contains_key(&filename) {
            // let music = self.player.new_music(filename.clone());
            // match music {
            // Ok(chip_music) => {
            // cartridge.music_tracks.insert(filename.clone(), chip_music);
            // cartridge.music_tracks_name.push(filename.clone());
            // }
            // Err(e) => {
            // error!("ERROR to load the music {:?}", e);
            // return -1;
            // }
            // }
            // }
            // }
            // cartridge.music_tracks.len() as i32 - 1
            0
        }

        pub fn new_sfx(&mut self, cartridge: &mut UnicornCartridge, filename: String) -> i32 {

            if filename != "" {
                if !cartridge.sound_tracks.contains_key(&filename) {
                    let sound = self.player.new_sound(filename.clone());
                    match sound {
                        Ok(chip_sound) => {
                            cartridge.sound_tracks.insert(filename.clone(), chip_sound);
                            cartridge.sound_tracks_name.push(filename.clone());
                        }
                        Err(e) => {
                            error!("ERROR to load the song {:?}", e);
                            return -1;
                        }
                    }
                }
            }
            cartridge.sound_tracks.len() as i32 - 1
        }

        pub fn sfx(&mut self,
                   cartridge: &mut UnicornCartridge,
                   _: Arc<Mutex<Sound>>,
                   id: i32,
                   filename: String,
                   channel: i32,
                   note: u16,
                   panning: i32,
                   rate: i32,
                   loops: i32)
                   -> i32 {

            debug!("PLAY SFX {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                   cartridge,
                   id,
                   filename,
                   channel,
                   note,
                   panning,
                   rate,
                   loops);

            let mut res = -1;

            if filename != "" {
                if !cartridge.sound_tracks.contains_key(&filename) {
                    let sound = self.player.load_sound(filename.clone());
                    match sound {
                        Ok(chip_sound) => {
                            cartridge.sound_tracks.insert(filename.clone(), chip_sound);
                            cartridge.sound_tracks_name.push(filename.clone());
                        }
                        Err(e) => error!("ERROR to load the song {:?}", e),
                    }
                }

                match cartridge.sound_tracks.get_mut(&filename) {
                    Some(mut sound) => {
                        res = self.player.play_sound(&mut sound, channel, note, panning, rate);
                    }
                    None => {}
                }
            }

            if id >= 0 {
                match cartridge.sound_tracks.get_mut(&cartridge.sound_tracks_name[id as usize]) {
                    Some(mut sound) => {
                        res = self.player.play_sound(&mut sound, channel, note, panning, rate);
                    }
                    None => {}
                }
            }

            res
        }

        pub fn update(&mut self, cartridge: &mut UnicornCartridge, sound: Arc<Mutex<Sound>>) {
            for sound_packet in self.crecv.try_iter() {
                debug!("[SOUND] PACKET {:?}", sound_packet);
                match packet::read_packet(sound_packet).unwrap() {
                    packet::Packet::ChiptuneMusic(res) => {
                        let filename = res.filename.clone();
                        // New music -> Load it before
                        let song = self.player.load_music(filename.clone());
                        match song {
                            Ok(chip_song) => {
                                if cartridge.music_track.len() == 0 {
                                    cartridge.music_track.push(chip_song);
                                } else {
                                    cartridge.music_track[0] = chip_song;
                                }
                            }
                            Err(e) => error!("ERROR to load the music {:?}", e),
                        }

                        match cartridge.music_track.get_mut(0) {
                            Some(mut song) => {
                                for i in 0..self.player.get_num_songs(&mut song) {
                                    let instru = self.player.get_song(&mut song, i).unwrap();
                                    let instru_name = self.player.get_name(instru);
                                    let name = format!("{:?}:{}", i, instru_name.clone());

                                    cartridge.sound_tracks.insert(name.clone(), instru);
                                    cartridge.sound_tracks_name.push(name.clone());
                                }
                            }
                            None => {}
                        }
                        // Play it
                        match cartridge.music_track.get_mut(0) {
                            Some(mut song) => {
                                self.player.play_music(&mut song, res.start_position);
                                self.player.set_looping(res.loops);
                            }
                            None => {}
                        }
                    }
                    packet::Packet::ChiptuneLoadSFX(res) => {
                        info!("LOAD SFX {:?}", res);

                        let filename = res.filename.clone();
                        if !cartridge.sound_tracks.contains_key(&filename) {
                            let sound = self.player.load_sound_from_memory(res.data.clone());
                            match sound {
                                Ok(chip_sound) => {
                                    cartridge.sound_tracks.insert(filename.clone(), chip_sound);
                                    cartridge.sound_tracks_name.push(filename.clone());
                                }

                                Err(e) => error!("ERROR to load the song {:?}", e),
                            }
                        }
                    }
                    packet::Packet::ChiptuneSFX(res) => {
                        debug!("PLAY SFX {:?}", res);

                        if res.filename != "" {
                            let filename = res.filename.clone();

                            if !cartridge.sound_tracks.contains_key(&filename) {
                                let sound = self.player.load_sound(filename.clone());
                                match sound {
                                    Ok(chip_sound) => {
                                        cartridge.sound_tracks.insert(filename.clone(), chip_sound);
                                        cartridge.sound_tracks_name.push(filename.clone());
                                    }

                                    Err(e) => error!("ERROR to load the song {:?}", e),
                                }
                            }

                            match cartridge.sound_tracks.get_mut(&filename) {
                                Some(mut sound) => {
                                    self.player.play_sound(&mut sound,
                                                           res.channel,
                                                           res.note,
                                                           res.panning,
                                                           res.rate);
                                }
                                None => {}
                            }
                        }

                        if res.id >= 0 && res.id < cartridge.sound_tracks_name.len() as i32 {
                            match cartridge.sound_tracks
                                .get_mut(&cartridge.sound_tracks_name[res.id as usize]) {
                                Some(mut sound) => {
                                    self.player.play_sound(&mut sound,
                                                           res.channel,
                                                           res.note,
                                                           res.panning,
                                                           res.rate);
                                }
                                None => {}
                            }
                        }
                    }
                    packet::Packet::ChiptuneMusicState(res) => {
                        if res.stop {
                            if res.chan >= 0 {
                                self.player.stop_chan(res.chan);
                            } else {
                                self.player.stop();
                            }
                        } else if res.pause {
                            self.player.pause(1);
                        } else if res.resume {
                            self.player.pause(0);
                        }
                    }

                    packet::Packet::ChiptuneVolume(res) => {
                        self.player.set_volume(res.volume);
                    }
                }
            }

            sound.lock().unwrap().chiptune_position = self.player.get_music_position();
        }
    }

    pub struct Sound {
        csend: mpsc::Sender<Vec<u8>>,
        chiptune_position: i32,
    }

    impl Sound {
        pub fn new(csend: mpsc::Sender<Vec<u8>>) -> Sound {
            Sound {
                csend: csend,
                chiptune_position: 0,
            }
        }

        // Chiptune
        pub fn music(&mut self,
                     id: i32,
                     filename: String,
                     channel: i32,
                     loops: i32,
                     start_position: i32) {
            debug!("[SOUND] Chiptune Music PLAY {:?}", filename);
            let p = packet::ChiptuneMusic {
                id: id,
                channel: channel,
                filename: filename,
                loops: loops,
                start_position: start_position,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn sfx(&mut self,
                   id: i32,
                   filename: String,
                   channel: i32,
                   note: u16,
                   panning: i32,
                   rate: i32,
                   loops: i32) {
            debug!("[SOUND] Chiptune SFX Play {:?}", id);
            let p = packet::ChiptuneSFX {
                id: id,
                filename: filename,
                channel: channel,
                loops: loops,
                note: note,
                panning: panning,
                rate: rate,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn load_sfx(&mut self, filename: String, data: Vec<u8>) {
            debug!("[SOUND] Chiptune SFX Load {:?}", filename);
            let p = packet::ChiptuneLoadSFX {
                filename: filename,
                data: data,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn music_stop(&mut self) {
            debug!("[SOUND] Chiptune STOP");
            let p = packet::ChiptuneMusicState {
                stop: true,
                chan: -1,
                pause: false,
                resume: false,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn stop_chan(&mut self, chan: i32) {
            debug!("[SOUND] Chiptune STOP CHAN");
            let p = packet::ChiptuneMusicState {
                stop: true,
                chan: chan,
                pause: false,
                resume: false,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn music_pause(&mut self) {
            debug!("[SOUND] Chiptune Pause");
            let p = packet::ChiptuneMusicState {
                stop: false,
                chan: -1,
                pause: true,
                resume: false,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn music_resume(&mut self) {
            debug!("[SOUND] Chiptune Resume");
            let p = packet::ChiptuneMusicState {
                stop: false,
                chan: -1,
                pause: false,
                resume: true,
            };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn music_volume(&mut self, volume: i32) {
            debug!("[SOUND] Chiptune volume");
            let p = packet::ChiptuneVolume { volume: volume };
            self.csend.send(packet::write_packet(p).unwrap()).unwrap();
        }

        pub fn chiptune_get_position(&mut self) -> i32 {
            self.chiptune_position
        }
    }
}