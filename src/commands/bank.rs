use crate::app::App;
use crate::commands::{Command, CommandRegistry, CommandResult};
use crate::model::ServerType;

pub struct AccountInfoCommand;

impl Command for AccountInfoCommand {
    fn name(&self) -> &'static str {
        "account_info"
    }

    fn description(&self) -> &'static str {
        "Displays information about a bank account on the connected server."
    }

    fn usage(&self) -> &'static str {
        "account_info <account_number>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        let Some(account_number) = args.first() else {
            app.log(format!("Usage: {}", self.usage()));
            return CommandResult::Ok;
        };

        let Some(target_ip) = &app.connection.target_ip else {
            app.log("Error: Not connected to any server.");
            return CommandResult::Ok;
        };

        let Some(server) = app.world.servers.get(target_ip) else {
            app.log("Error: Not connected to any server.");
            return CommandResult::Ok;
        };

        if server.server_type != ServerType::Bank {
            app.log("Error: Not connected to a bank server.");
            return CommandResult::Ok;
        }

        if server.is_locked {
            app.log("Access Denied: Server Locked. Cannot view accounts.");
            return CommandResult::Ok;
        }

        let Some(accounts) = &server.accounts else {
            app.log("Error: No accounts found on this bank server.");
            return CommandResult::Ok;
        };

        let Some(account) = accounts
            .iter()
            .find(|a| a.account_number == *account_number)
        else {
            app.log(format!(
                "Error: Account {} not found on this server.",
                account_number
            ));
            return CommandResult::Ok;
        };

        // Extract data before releasing the borrow
        let account_num = account.account_number.clone();
        let owner = account.owner.clone();
        let balance = account.balance;

        app.log(format!("--- Account Info for {} ---", account_num));
        app.log(format!("Owner: {}", owner));
        app.log(format!("Balance: {}c", balance));
        app.log("----------------------------");

        CommandResult::Ok
    }
}

pub struct TransferCommand;

impl Command for TransferCommand {
    fn name(&self) -> &'static str {
        "transfer"
    }

    fn description(&self) -> &'static str {
        "Transfers credits between bank accounts on the connected server."
    }

    fn usage(&self) -> &'static str {
        "transfer <from_account_number> <to_account_number> <amount>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        if args.len() != 3 {
            app.log(format!("Usage: {}", self.usage()));
            return CommandResult::Ok;
        }

        let from_account_number = args[0];
        let to_account_number = args[1];

        let amount: i32 = match args[2].parse() {
            Ok(amt) if amt > 0 => amt,
            _ => {
                app.log("Error: Invalid transfer amount. Must be a positive number.");
                return CommandResult::Ok;
            }
        };

        let Some(target_ip) = app.connection.target_ip.clone() else {
            app.log("Error: Not connected to any server.");
            return CommandResult::Ok;
        };

        // Perform transfer and collect result message
        let result_msg = {
            let Some(server) = app.world.servers.get_mut(&target_ip) else {
                return log_and_return(app, "Error: Not connected to any server.");
            };

            if server.server_type != ServerType::Bank {
                return log_and_return(app, "Error: Not connected to a bank server.");
            }

            if server.is_locked {
                return log_and_return(app, "Access Denied: Server Locked. Cannot transfer funds.");
            }

            let Some(accounts) = &mut server.accounts else {
                return log_and_return(app, "Error: No accounts found on this bank server.");
            };

            let Some(from_idx) = accounts
                .iter()
                .position(|a| a.account_number == from_account_number)
            else {
                return log_and_return(
                    app,
                    &format!(
                        "Error: Sender account {} not found on this server.",
                        from_account_number
                    ),
                );
            };

            let Some(to_idx) = accounts
                .iter()
                .position(|a| a.account_number == to_account_number)
            else {
                return log_and_return(
                    app,
                    &format!(
                        "Error: Receiver account {} not found on this server.",
                        to_account_number
                    ),
                );
            };

            if from_idx == to_idx {
                return log_and_return(app, "Error: Cannot transfer to the same account.");
            }

            // Use split_at_mut to get two mutable references to different elements
            let (first_part, second_part) = accounts.split_at_mut(std::cmp::max(from_idx, to_idx));
            let (from_account, to_account) = if from_idx < to_idx {
                (&mut first_part[from_idx], &mut second_part[0])
            } else {
                (&mut second_part[0], &mut first_part[to_idx])
            };

            if from_account.balance < amount {
                return log_and_return(
                    app,
                    &format!(
                        "Error: Insufficient funds in account {}.",
                        from_account_number
                    ),
                );
            }

            from_account.balance -= amount;
            to_account.balance += amount;

            format!(
                "Transfer successful: {}c from {} (new balance: {}c) to {} (new balance: {}c).",
                amount,
                from_account_number,
                from_account.balance,
                to_account_number,
                to_account.balance
            )
        };

        app.log(result_msg);
        CommandResult::Ok
    }
}

fn log_and_return(app: &mut App, msg: &str) -> CommandResult {
    app.log(msg);
    CommandResult::Ok
}
