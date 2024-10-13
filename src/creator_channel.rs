use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatorChannelConfig {
    pub(crate) guild_id: GuildId,
    pub(crate) creator_id: ChannelId,
    pub(crate) category_id: ChannelId,
    pub(crate) naming_standard: String,
    pub(crate) channel_numbers: HashSet<u16>,
    pub(crate) user_limit:u32
}

impl CreatorChannelConfig {
    pub(crate) fn get_next_number(&self) -> u16 {
        get_next_number(self, 1)
    }

    pub(crate) fn add_number(&mut self, number: u16) -> bool {
        self.channel_numbers.insert(number)
    }

    pub(crate) fn remove_number(&mut self, number: &u16) -> bool {
        self.channel_numbers.remove(number)
    }

    pub(crate) fn get_highest_number(&self) -> Option<u16> {
        let length = self.channel_numbers.len();

        return if length == 0 {
            None
        } else {
            Some(get_highest_number(self, length as u16))
        }
    }
}

fn get_next_number(creator_channel_config: &CreatorChannelConfig, number: u16) -> u16 {
    if creator_channel_config.channel_numbers.contains(&number) {
        get_next_number(creator_channel_config, number+1)
    } else {
        return number
    }
}

fn get_highest_number(creator_channel_config: &CreatorChannelConfig, number: u16) -> u16 {
    if creator_channel_config.channel_numbers.contains(&number) {
        number
    } else {
        get_highest_number(creator_channel_config, number-1)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::creator_channel::CreatorChannelConfig;

    #[test]
    fn check_channel_numbers() {
        let mut creator_channel_config = CreatorChannelConfig {
            guild_id: Default::default(),
            creator_id: Default::default(),
            category_id: Default::default(),
            naming_standard: "".to_string(),
            channel_numbers: HashSet::new(),
            user_limit: 0,
        };

        let number_1 = creator_channel_config.get_next_number();
        assert_eq!(number_1, 1);
        creator_channel_config.add_number(number_1);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), 1);

        let number_2 = creator_channel_config.get_next_number();
        assert_eq!(number_2, 2);
        creator_channel_config.add_number(number_2);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), 2);

        let number_3 = creator_channel_config.get_next_number();
        assert_eq!(number_3, 3);
        creator_channel_config.add_number(number_3);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), 3);

        creator_channel_config.remove_number(&number_2);

        let number_2 = creator_channel_config.get_next_number();
        assert_eq!(number_2, 2);
        creator_channel_config.add_number(number_2);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), 3);

        creator_channel_config.remove_number(&number_2);
        creator_channel_config.remove_number(&number_1);

        let number_1 = creator_channel_config.get_next_number();
        assert_eq!(number_1, 1);
        creator_channel_config.add_number(number_1);

        let number_2 = creator_channel_config.get_next_number();
        assert_eq!(number_2, 2);
        creator_channel_config.add_number(number_2);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), 3);
    }
}