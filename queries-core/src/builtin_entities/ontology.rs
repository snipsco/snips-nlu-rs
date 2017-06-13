use std::convert::From;

use errors::*;
use rustling_ontology::{DimensionKind, Grain as RustlingGrain};
use rustling_ontology::dimension::{Precision as RustlingPrecision};
use rustling_ontology::output::{IntegerOutput, FloatOutput, OrdinalOutput, TimeOutput,
                                TimeIntervalOutput, AmountOfMoneyOutput, TemperatureOutput,
                                DurationOutput, Output};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content="value")]
pub enum BuiltinEntity {
    Number(NumberValue),
    Ordinal(OrdinalValue),
    Time(TimeValue),
    AmountOfMoney(AmountOfMoneyValue),
    Temperature(TemperatureValue),
    Duration(DurationValue),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct NumberValue(pub f64);

impl From<IntegerOutput> for NumberValue {
    fn from(v: IntegerOutput) -> NumberValue {
        NumberValue(v.0 as f64)
    }
}

impl From<FloatOutput> for NumberValue {
    fn from(v: FloatOutput) -> NumberValue {
        NumberValue(v.0 as f64)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct OrdinalValue(pub i64);

impl From<OrdinalOutput> for OrdinalValue {
    fn from(v: OrdinalOutput) -> OrdinalValue {
        OrdinalValue(v.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "kind", content="value")]
pub enum TimeValue {
    InstantTime(InstantTimeValue),
    TimeInterval(TimeIntervalValue)
}

impl From<TimeOutput> for TimeValue {
    fn from(v: TimeOutput) -> TimeValue {
        TimeValue::InstantTime(InstantTimeValue::from(v))
    }
}

impl From<TimeIntervalOutput> for TimeValue {
    fn from(v: TimeIntervalOutput) -> TimeValue {
        TimeValue::TimeInterval(TimeIntervalValue::from(v))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct InstantTimeValue {
    pub value: String,
    pub grain: Grain,
    pub precision: Precision,
}

impl From<TimeOutput> for InstantTimeValue {
    fn from(v: TimeOutput) -> InstantTimeValue {
        InstantTimeValue {
            value: v.moment.to_string(),
            grain: Grain::from(v.grain),
            precision: Precision::from(v.precision)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct TimeIntervalValue {
    pub from: Option<String>,
    pub to: Option<String>
}

impl From<TimeIntervalOutput> for TimeIntervalValue {
    fn from(v: TimeIntervalOutput) -> TimeIntervalValue {
        match v {
            TimeIntervalOutput::After(after) => TimeIntervalValue {
                from: Some(after.to_string()),
                to: None
            },
            TimeIntervalOutput::Before(before) => TimeIntervalValue {
                from: None,
                to: Some(before.to_string())
            },
            TimeIntervalOutput::Between(from, to) => TimeIntervalValue {
                from: Some(from.to_string()),
                to: Some(to.to_string())
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct AmountOfMoneyValue {
    pub value: f32,
    pub precision: Precision,
    pub unit: Option<&'static str>,
}

impl From<AmountOfMoneyOutput> for AmountOfMoneyValue {
    fn from(v: AmountOfMoneyOutput) -> AmountOfMoneyValue {
        AmountOfMoneyValue { value: v.value, precision: Precision::from(v.precision), unit: v.unit }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct TemperatureValue {
    pub value: f32,
    pub unit: Option<&'static str>,
}

impl From<TemperatureOutput> for TemperatureValue {
    fn from(v: TemperatureOutput) -> TemperatureValue {
        TemperatureValue { value: v.value, unit: v.unit }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct DurationValue {
    pub years: i64,
    pub quarters: i64,
    pub months: i64,
    pub weeks: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub precision: Precision,
}

impl From<DurationOutput> for DurationValue {
    fn from(v: DurationOutput) -> DurationValue {
        let mut years: i64 = 0;
        let mut quarters: i64 = 0;
        let mut months: i64 = 0;
        let mut weeks: i64 = 0;
        let mut days: i64 = 0;
        let mut hours: i64 = 0;
        let mut minutes: i64 = 0;
        let mut seconds: i64 = 0;
        for comp in v.period.comps().iter() {
            match comp.grain {
                RustlingGrain::Year => years = comp.quantity,
                RustlingGrain::Quarter => quarters = comp.quantity,
                RustlingGrain::Month => months = comp.quantity,
                RustlingGrain::Week => weeks = comp.quantity,
                RustlingGrain::Day => days = comp.quantity,
                RustlingGrain::Hour => hours = comp.quantity,
                RustlingGrain::Minute => minutes = comp.quantity,
                RustlingGrain::Second => seconds = comp.quantity,
            }
        }

        DurationValue {
            years,
            quarters,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            precision: Precision::from(v.precision)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Grain {
    Year = 0,
    Quarter = 1,
    Month = 2,
    Week = 3,
    Day = 4,
    Hour = 5,
    Minute = 6,
    Second = 7,
}

impl From<RustlingGrain> for Grain {
    fn from(v: RustlingGrain) -> Grain {
        match v {
            RustlingGrain::Year => Grain::Year,
            RustlingGrain::Quarter => Grain::Quarter,
            RustlingGrain::Month => Grain::Month,
            RustlingGrain::Week => Grain::Week,
            RustlingGrain::Day => Grain::Day,
            RustlingGrain::Hour => Grain::Hour,
            RustlingGrain::Minute => Grain::Minute,
            RustlingGrain::Second => Grain::Second,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum Precision {
    Approximate,
    Exact,
}

impl From<RustlingPrecision> for Precision {
    fn from(v: RustlingPrecision) -> Precision {
        match v {
            RustlingPrecision::Approximate => Precision::Approximate,
            RustlingPrecision::Exact => Precision::Exact,
        }
    }
}

impl BuiltinEntity {
    pub fn from_rustling_output(o: &Output) -> BuiltinEntity {
        match *o {
            Output::AmountOfMoney(ref v) => BuiltinEntity::AmountOfMoney(AmountOfMoneyValue::from(v.clone())),
            Output::Duration(ref v) => BuiltinEntity::Duration(DurationValue::from(v.clone())),
            Output::Float(ref v) => BuiltinEntity::Number(NumberValue::from(v.clone())),
            Output::Integer(ref v) => BuiltinEntity::Number(NumberValue::from(v.clone())),
            Output::Ordinal(ref v) => BuiltinEntity::Ordinal(OrdinalValue::from(v.clone())),
            Output::Temperature(ref v) => BuiltinEntity::Temperature(TemperatureValue::from(v.clone())),
            Output::Time(ref v) => BuiltinEntity::Time(TimeValue::from(v.clone())),
            Output::TimeInterval(ref v) => BuiltinEntity::Time(TimeValue::from(v.clone())),
        }
    }
}


#[derive(Copy, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuiltinEntityKind {
    AmountOfMoney,
    Duration,
    Number,
    Ordinal,
    Temperature,
    Time,
}

impl BuiltinEntityKind {
    pub fn all() -> Vec<BuiltinEntityKind> {
        vec![
            BuiltinEntityKind::AmountOfMoney,
            BuiltinEntityKind::Duration,
            BuiltinEntityKind::Number,
            BuiltinEntityKind::Ordinal,
            BuiltinEntityKind::Temperature,
            BuiltinEntityKind::Time,
        ]
    }
}

impl BuiltinEntityKind {
    pub fn identifier(&self) -> &str {
        match *self {
            BuiltinEntityKind::AmountOfMoney => "snips/amountOfMoney",
            BuiltinEntityKind::Duration => "snips/duration",
            BuiltinEntityKind::Number => "snips/number",
            BuiltinEntityKind::Ordinal => "snips/ordinal",
            BuiltinEntityKind::Temperature => "snips/temperature",
            BuiltinEntityKind::Time => "snips/datetime",
        }
    }

    pub fn from_identifier(identifier: &str) -> Result<BuiltinEntityKind> {
        Self::all()
            .into_iter()
            .find(|kind| kind.identifier() == identifier)
            .ok_or(format!("Unknown EntityKind identifier: {}", identifier).into())
    }

    pub fn from_rustling_output(v: &Output) -> BuiltinEntityKind {
        match *v {
            Output::AmountOfMoney(_) => BuiltinEntityKind::AmountOfMoney,
            Output::Duration(_) => BuiltinEntityKind::Duration,
            Output::Float(_) => BuiltinEntityKind::Number,
            Output::Integer(_) => BuiltinEntityKind::Number,
            Output::Ordinal(_) => BuiltinEntityKind::Ordinal,
            Output::Temperature(_) => BuiltinEntityKind::Temperature,
            Output::Time(_) => BuiltinEntityKind::Time,
            Output::TimeInterval(_) => BuiltinEntityKind::Time,
        }
    }

    pub fn dimension_kind(&self) -> DimensionKind {
        match *self {
            BuiltinEntityKind::AmountOfMoney => DimensionKind::AmountOfMoney,
            BuiltinEntityKind::Duration => DimensionKind::Duration,
            BuiltinEntityKind::Number => DimensionKind::Number,
            BuiltinEntityKind::Ordinal => DimensionKind::Ordinal,
            BuiltinEntityKind::Temperature => DimensionKind::Temperature,
            BuiltinEntityKind::Time => DimensionKind::Time,
        }
    }
}
