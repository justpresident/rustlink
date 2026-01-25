use super::Tool;
use crate::app::App;

pub struct PasswordBreaker;

impl Tool for PasswordBreaker {
    fn name(&self) -> &'static str {
        "PasswordBreaker"
    }

    fn description(&self) -> &'static str {
        "Cracks password-protected servers"
    }

    fn can_run(&self, app: &App, target_ip: &str) -> Result<(), String> {
        let Some(server) = app.world.get(target_ip) else {
            return Err("Target server not found.".into());
        };

        if !server.is_locked {
            return Err("Server is not locked. No need for PasswordBreaker.".into());
        }

        // Check if firewall is blocking
        if let Some(firewall) = &server.firewall
            && firewall.is_active
        {
            return Err("Firewall is active. Disable it first with FirewallBuster.".into());
        }

        Ok(())
    }

    fn on_tick(&self, _app: &App, _target_ip: &str, current_progress: f64) -> f64 {
        // PasswordBreaker runs at moderate speed
        (current_progress + 2.5).min(100.0)
    }

    fn on_complete(&self, app: &mut App, target_ip: &str) {
        if let Some(server) = app.world.get_mut(target_ip) {
            server.is_locked = false;
            app.log(format!("SUCCESS: Password cracked on {target_ip}"));
        }
    }
}
