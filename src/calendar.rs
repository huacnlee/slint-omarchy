use chrono::{Datelike, NaiveDate, Weekday};

pub struct MonthPage {
    pub year: i32,
    pub month: u32,
    pub label: String,
    pub leading: u32,
    pub days: u32,
    pub previous_days: u32,
}

impl MonthPage {
    pub fn new(year: i32, month: u32) -> Option<Self> {
        let first = NaiveDate::from_ymd_opt(year, month, 1)?;
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let next = NaiveDate::from_ymd_opt(next_year, next_month, 1)?;
        let days = next.signed_duration_since(first).num_days() as u32;
        Some(Self {
            year,
            month,
            label: first.format("%B").to_string(),
            leading: first.weekday().num_days_from_sunday(),
            days,
            previous_days: first.pred_opt()?.day(),
        })
    }

    pub fn shifted(&self, delta: i32) -> Option<Self> {
        let index = self
            .year
            .checked_mul(12)?
            .checked_add(self.month as i32 - 1)?
            .checked_add(delta)?;
        let year = index.div_euclid(12);
        let month = index.rem_euclid(12) as u32 + 1;
        Self::new(year, month)
    }

    pub fn selectable_date(&self, day: u32) -> Option<NaiveDate> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, day)?;
        match date.weekday() {
            Weekday::Sat | Weekday::Sun => None,
            _ => Some(date),
        }
    }

    pub fn selectable_offset_date(&self, offset: i32, day: u32) -> Option<NaiveDate> {
        self.shifted(offset)?.selectable_date(day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_year_and_weekday_rules() {
        let february = MonthPage::new(2028, 2).unwrap();
        assert_eq!(february.days, 29);
        assert_eq!(february.leading, 2);
        assert_eq!(february.previous_days, 31);
        assert!(february.selectable_date(29).is_some());
        assert!(february.selectable_date(26).is_none());
        assert!(february.selectable_date(30).is_none());
    }

    #[test]
    fn shifting_crosses_year_boundary() {
        let december = MonthPage::new(2026, 12).unwrap();
        let january = december.shifted(1).unwrap();
        assert_eq!((january.year, january.month), (2027, 1));
        assert_eq!(
            (
                january.shifted(-1).unwrap().year,
                january.shifted(-1).unwrap().month
            ),
            (2026, 12)
        );
    }

    #[test]
    fn adjacent_month_dates_respect_weekday_rules() {
        let january = MonthPage::new(2027, 1).unwrap();
        assert_eq!(january.leading, 5);
        assert_eq!(january.previous_days, 31);
        assert_eq!(
            january.selectable_offset_date(-1, 31),
            NaiveDate::from_ymd_opt(2026, 12, 31)
        );
        assert_eq!(
            january.selectable_offset_date(1, 1),
            NaiveDate::from_ymd_opt(2027, 2, 1)
        );
        assert!(january.selectable_offset_date(-1, 27).is_none());
    }
}
