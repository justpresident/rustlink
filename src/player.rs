use crate::model::{File, Mail, Mission};

/// Player state for economy, inventory, and missions
pub struct Player {
    pub credits: u32,
    pub local_files: Vec<File>,
    pub missions: Vec<Mission>,
    pub inbox: Vec<Mail>,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Self {
            credits: 500,
            local_files: Vec::new(),
            missions: vec![
                Mission {
                    id: 1,
                    description: "Steal 'research.doc' from Central Data".into(),
                    target_ip: "172.16.0.4".into(),
                    target_file: "research.doc".into(),
                    reward: 2000,
                    is_complete: false,
                },
                Mission {
                    id: 2,
                    description: "Download 'accounts.dat' from Global Trust Bank".into(),
                    target_ip: "212.43.10.5".into(),
                    target_file: "accounts.dat".into(),
                    reward: 5000,
                    is_complete: false,
                },
            ],
            inbox: vec![
                Mail {
                    id: 1,
                    sender: "admin@uplink.net".into(),
                    subject: "Welcome to Uplink!".into(),
                    body: "Welcome, Agent. Your journey into the digital underworld begins now. Good luck.".into(),
                    is_read: false,
                },
                Mail {
                    id: 2,
                    sender: "intern@globaltrust.com".into(),
                    subject: "Urgent: System Vulnerability".into(),
                    body: "We've detected a critical vulnerability in our systems. Please assist immediately.".into(),
                    is_read: false,
                },
            ],
        }
    }

    pub fn add_file(&mut self, file: File) {
        self.local_files.push(file);
    }

    pub fn penalize(&mut self, amount: u32) {
        self.credits = self.credits.saturating_sub(amount);
    }

    /// Check and complete missions for a downloaded file, returns list of rewards
    pub fn check_missions(&mut self, filename: &str) -> Vec<u32> {
        let mut rewards = Vec::new();
        for m in self.missions.iter_mut() {
            if !m.is_complete && m.target_file == filename {
                m.is_complete = true;
                self.credits += m.reward;
                rewards.push(m.reward);
            }
        }
        rewards
    }

    pub fn unread_mail_count(&self) -> usize {
        self.inbox.iter().filter(|m| !m.is_read).count()
    }
}
