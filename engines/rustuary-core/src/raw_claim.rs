use crate::error::{ActuarialError, Result};

/// Calendar date carried by canonical claim/event build records.
///
/// The Python adapter resolves source columns and constants before creating
/// these dates. The Rust core keeps the date typed so later triangle
/// construction can own origin bucketing and development-age calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordDate {
    year: i32,
    month: u8,
    day: u8,
}

impl RecordDate {
    /// Construct a validated proleptic Gregorian record date.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self> {
        if !is_valid_date(year, month, day) {
            return Err(ActuarialError::InvalidClaimEventDate { year, month, day });
        }

        Ok(Self { year, month, day })
    }

    /// Return the calendar year.
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }

    /// Return the one-based calendar month.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// Return the one-based day of month.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }
}

/// Ordered segment value resolved from a `TriangleDefinition`.
///
/// Segment order is retained by the surrounding record and later becomes part
/// of the deterministic triangle grouping key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentValue {
    name: String,
    value: String,
}

impl SegmentValue {
    /// Construct a validated segment name/value pair.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let value = value.into();
        validate_non_empty("segments.name", &name)?;
        validate_non_empty("segments.value", &value)?;

        Ok(Self { name, value })
    }

    /// Return the canonical segment name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the segment value for this record.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Input bag for [`ClaimEventRecord::new`].
///
/// Python, CLI, and service adapters resolve source columns and constants into
/// this canonical shape before handing the records to `rustuary-core`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimEventRecordInput {
    /// Source event date used to derive the origin period.
    pub origin_date: RecordDate,
    /// Source event date used with `origin_date` to derive development age.
    pub development_date: RecordDate,
    /// Monetary amount for `sum` aggregation; omitted for simple count records.
    pub amount: Option<f64>,
    /// Main reserving class / actuarial reserving unit.
    pub portfolio_id: String,
    /// Ordered segment values below `portfolio_id`.
    pub segments: Vec<SegmentValue>,
    /// Claims measure such as `paid`, `incurred`, or `reported_count`.
    pub measure: String,
    /// Optional valuation or data-cut date for audit context.
    pub valuation_date: Option<RecordDate>,
    /// Optional currency code for monetary records.
    pub currency: Option<String>,
}

/// Canonical claim/event record consumed by Rust triangle construction.
///
/// This is not a dataframe row and it does not retain source column names. It
/// is the typed boundary after adapter-level column mapping and before
/// deterministic Rust bucketing, grouping, aggregation, and basis conversion.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimEventRecord {
    origin_date: RecordDate,
    development_date: RecordDate,
    amount: Option<f64>,
    portfolio_id: String,
    segments: Vec<SegmentValue>,
    measure: String,
    valuation_date: Option<RecordDate>,
    currency: Option<String>,
}

impl ClaimEventRecord {
    /// Construct a validated canonical claim/event build record.
    pub fn new(input: ClaimEventRecordInput) -> Result<Self> {
        validate_non_empty("portfolio_id", &input.portfolio_id)?;
        validate_non_empty("measure", &input.measure)?;
        if let Some(currency) = &input.currency {
            validate_non_empty("currency", currency)?;
        }
        if let Some(amount) = input.amount {
            if !amount.is_finite() {
                return Err(ActuarialError::NonFiniteClaimEventAmount);
            }
        }

        Ok(Self {
            origin_date: input.origin_date,
            development_date: input.development_date,
            amount: input.amount,
            portfolio_id: input.portfolio_id,
            segments: input.segments,
            measure: input.measure,
            valuation_date: input.valuation_date,
            currency: input.currency,
        })
    }

    /// Return the date used to derive the origin period.
    #[must_use]
    pub const fn origin_date(&self) -> RecordDate {
        self.origin_date
    }

    /// Return the date used to derive development age.
    #[must_use]
    pub const fn development_date(&self) -> RecordDate {
        self.development_date
    }

    /// Return the optional monetary amount.
    #[must_use]
    pub const fn amount(&self) -> Option<f64> {
        self.amount
    }

    /// Return the main reserving class / actuarial reserving unit.
    #[must_use]
    pub fn portfolio_id(&self) -> &str {
        &self.portfolio_id
    }

    /// Return ordered segment values.
    #[must_use]
    pub fn segments(&self) -> &[SegmentValue] {
        &self.segments
    }

    /// Return the claims measure.
    #[must_use]
    pub fn measure(&self) -> &str {
        &self.measure
    }

    /// Return optional valuation or data-cut date.
    #[must_use]
    pub const fn valuation_date(&self) -> Option<RecordDate> {
        self.valuation_date
    }

    /// Return optional currency code.
    #[must_use]
    pub fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ActuarialError::EmptyClaimEventRecordField { field });
    }
    Ok(())
}

fn is_valid_date(year: i32, month: u8, day: u8) -> bool {
    if year <= 0 || month == 0 || month > 12 || day == 0 {
        return false;
    }

    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    day <= max_day
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::{ClaimEventRecord, ClaimEventRecordInput, RecordDate, SegmentValue};
    use crate::ActuarialError;

    fn sample_record_input() -> ClaimEventRecordInput {
        ClaimEventRecordInput {
            origin_date: RecordDate::new(2024, 1, 15).expect("sample origin date should be valid"),
            development_date: RecordDate::new(2024, 3, 10)
                .expect("sample development date should be valid"),
            amount: Some(100.0),
            portfolio_id: "Motor".to_owned(),
            segments: vec![
                SegmentValue::new("country", "CH").expect("sample segment should be valid")
            ],
            measure: "paid".to_owned(),
            valuation_date: Some(
                RecordDate::new(2026, 12, 31).expect("sample valuation date should be valid"),
            ),
            currency: Some("CHF".to_owned()),
        }
    }

    #[test]
    fn claim_event_record_preserves_canonical_values() {
        let record =
            ClaimEventRecord::new(sample_record_input()).expect("sample record should be valid");

        assert_eq!(
            record.origin_date(),
            RecordDate::new(2024, 1, 15).expect("assertion date should be valid")
        );
        assert_eq!(
            record.development_date(),
            RecordDate::new(2024, 3, 10).expect("assertion date should be valid")
        );
        assert_eq!(record.amount(), Some(100.0));
        assert_eq!(record.portfolio_id(), "Motor");
        assert_eq!(record.segments()[0].name(), "country");
        assert_eq!(record.segments()[0].value(), "CH");
        assert_eq!(record.measure(), "paid");
        assert_eq!(
            record.valuation_date(),
            Some(RecordDate::new(2026, 12, 31).expect("assertion date should be valid"))
        );
        assert_eq!(record.currency(), Some("CHF"));
    }

    #[test]
    fn claim_event_record_allows_count_records_without_amount() {
        let mut input = sample_record_input();
        input.amount = None;
        "reported_count".clone_into(&mut input.measure);

        let record = ClaimEventRecord::new(input).expect("count record should be valid");

        assert_eq!(record.amount(), None);
        assert_eq!(record.measure(), "reported_count");
    }

    #[test]
    fn record_date_validates_calendar_dates() {
        assert_eq!(
            RecordDate::new(2024, 2, 29)
                .expect("leap day should be valid")
                .day(),
            29
        );
        assert_eq!(
            RecordDate::new(2023, 2, 29).expect_err("non-leap day should fail"),
            ActuarialError::InvalidClaimEventDate {
                year: 2023,
                month: 2,
                day: 29,
            }
        );
        assert_eq!(
            RecordDate::new(2024, 13, 1).expect_err("month 13 should fail"),
            ActuarialError::InvalidClaimEventDate {
                year: 2024,
                month: 13,
                day: 1,
            }
        );
        assert_eq!(
            RecordDate::new(0, 1, 1).expect_err("year zero should fail"),
            ActuarialError::InvalidClaimEventDate {
                year: 0,
                month: 1,
                day: 1,
            }
        );
    }

    #[test]
    fn claim_event_record_rejects_non_finite_amounts() {
        let mut input = sample_record_input();
        input.amount = Some(f64::NAN);

        assert_eq!(
            ClaimEventRecord::new(input).expect_err("non-finite amount should fail"),
            ActuarialError::NonFiniteClaimEventAmount
        );
    }

    #[test]
    fn claim_event_record_rejects_blank_grouping_fields() {
        let mut input = sample_record_input();
        " ".clone_into(&mut input.portfolio_id);

        assert_eq!(
            ClaimEventRecord::new(input).expect_err("blank portfolio should fail"),
            ActuarialError::EmptyClaimEventRecordField {
                field: "portfolio_id"
            }
        );
    }

    #[test]
    fn segment_value_rejects_blank_names_and_values() {
        assert_eq!(
            SegmentValue::new(" ", "CH").expect_err("blank segment name should fail"),
            ActuarialError::EmptyClaimEventRecordField {
                field: "segments.name"
            }
        );
        assert_eq!(
            SegmentValue::new("country", " ").expect_err("blank segment value should fail"),
            ActuarialError::EmptyClaimEventRecordField {
                field: "segments.value"
            }
        );
    }
}
