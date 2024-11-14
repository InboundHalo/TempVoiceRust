use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId};
use std::collections::HashSet;
use std::num::{NonZero, NonZeroU16};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatorChannelConfig {
    pub(crate) guild_id: GuildId,
    pub(crate) creator_id: ChannelId,
    pub(crate) category_id: ChannelId,
    pub(crate) naming_standard: String,
    pub(crate) channel_numbers: HashSet<NonZeroU16>,
    pub(crate) user_limit: u32,
}

impl CreatorChannelConfig {
    pub(crate) fn get_next_number(&self) -> NonZeroU16 {
        get_next_number(self, NonZero::new(1).expect("This should never be 0"))
    }

    pub(crate) fn add_number(&mut self, number: NonZeroU16) -> bool {
        self.channel_numbers.insert(number)
    }

    pub(crate) fn remove_number(&mut self, number: &NonZeroU16) -> bool {
        self.channel_numbers.remove(number)
    }

    pub(crate) fn get_highest_number(&self) -> Option<NonZeroU16> {
        self.channel_numbers.iter().max().cloned()
    }
}

fn get_next_number(creator_channel_config: &CreatorChannelConfig, number: NonZeroU16) -> NonZeroU16 {
    if creator_channel_config.channel_numbers.contains(&number) {
        get_next_number(creator_channel_config, number.checked_add(1).expect("This is adding a positive number so should never == 0"))
    } else {
        return number;
    }
}

#[cfg(test)]
mod tests {
    use crate::creator_channel::CreatorChannelConfig;
    use std::collections::HashSet;
    use std::num::NonZeroU16;

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
        assert_eq!(number_1, NonZeroU16::new(1).unwrap());
        creator_channel_config.add_number(number_1);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), NonZeroU16::new(1).unwrap());

        let number_2 = creator_channel_config.get_next_number();
        assert_eq!(number_2, NonZeroU16::new(2).unwrap());
        creator_channel_config.add_number(number_2);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), NonZeroU16::new(2).unwrap());

        let number_3 = creator_channel_config.get_next_number();
        assert_eq!(number_3, NonZeroU16::new(3).unwrap());
        creator_channel_config.add_number(number_3);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), NonZeroU16::new(3).unwrap());

        creator_channel_config.remove_number(&number_2);

        let number_2 = creator_channel_config.get_next_number();
        assert_eq!(number_2, NonZeroU16::new(2).unwrap());
        creator_channel_config.add_number(number_2);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), NonZeroU16::new(3).unwrap());

        creator_channel_config.remove_number(&number_2);
        creator_channel_config.remove_number(&number_1);

        let number_1 = creator_channel_config.get_next_number();
        assert_eq!(number_1, NonZeroU16::new(1).unwrap());
        creator_channel_config.add_number(number_1);

        let number_2 = creator_channel_config.get_next_number();
        assert_eq!(number_2, NonZeroU16::new(2).unwrap());
        creator_channel_config.add_number(number_2);

        let highest = creator_channel_config.get_highest_number();
        assert_eq!(highest.is_some(), true);
        assert_eq!(highest.unwrap(), NonZeroU16::new(3).unwrap());
    }
}
