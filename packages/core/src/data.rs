// Canonical data types used across crates
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Timestamp wrapper for clarity and type-safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn from_millis(millis: u64) -> Self {
        Timestamp(millis)
    }
    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Timestamp {
    fn from(v: u64) -> Self {
        Timestamp::from_millis(v)
    }
}

impl From<Timestamp> for u64 {
    fn from(t: Timestamp) -> Self {
        t.as_millis()
    }
}

/// Message delivered to UI representing bytes received/sent on a port
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    timestamp: Timestamp,
    direction: Direction,
    /// Message payload as a UTF-8 string (display-friendly)
    text: String,
}

impl Message {
    pub fn new(timestamp: Timestamp, direction: Direction, text: impl Into<String>) -> Self {
        Self {
            timestamp,
            direction,
            text: text.into(),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::In => write!(f, "In"),
            Direction::Out => write!(f, "Out"),
        }
    }
}

/// A single data point with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    timestamp: Timestamp,
    label: Option<String>,
    value: f64,
}

impl Point {
    /// Create a new Point. A timestamp is required to emphasize explicit
    /// time handling.
    pub fn new(timestamp: Timestamp, value: f64) -> Self {
        Self {
            timestamp,
            label: None,
            value,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Container for serial data with circular buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointBuffer(VecDeque<Point>);

impl PointBuffer {
    pub fn new(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }

    /// Add a new data point
    pub fn push(&mut self, point: Point) {
        self.0.push_back(point);
    }

    /// Add a value with an explicit timestamp
    pub fn push_value(&mut self, timestamp: Timestamp, value: f64) {
        self.push(Point::new(timestamp, value));
    }

    /// Get the latest data point
    pub fn last(&self) -> Option<&Point> {
        self.0.back()
    }

    /// Clear all data points
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Get the number of data points
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get an iterator over the data points
    pub fn iter(&self) -> impl Iterator<Item = &Point> {
        self.0.iter()
    }

    /// Get an iterator over data points matching the given name
    pub fn iter_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Point> + 'a {
        self.0
            .iter()
            .filter(move |p| p.label.as_deref() == Some(name))
    }

    /// Get a new DataBuffer containing only points with the given name
    pub fn filtered_by_name(&self, name: &str) -> Self {
        Self(
            self.0
                .iter()
                .filter(|p| p.label.as_deref() == Some(name))
                .cloned()
                .collect(),
        )
    }
}

impl Default for PointBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_point_creation() {
        let point = Point::new(Timestamp(0), 42.0);
        assert_eq!(point.value(), 42.0);
    }

    #[test]
    fn test_data_buffer_push() {
        let mut buffer = PointBuffer::new(10);
        buffer.push_value(Timestamp(1), 1.0);
        buffer.push_value(Timestamp(2), 2.0);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_data_buffer_last() {
        let mut buffer = PointBuffer::new(10);
        buffer.push_value(Timestamp(1), 1.0);
        buffer.push_value(Timestamp(2), 2.0);
        assert_eq!(buffer.last().unwrap().value(), 2.0);
    }

    #[test]
    fn test_data_buffer_filter_by_name() {
        let mut buffer = PointBuffer::new(10);
        buffer.push(Point::new(Timestamp(1), 1.0).with_label("temp"));
        buffer.push(Point::new(Timestamp(2), 2.0).with_label("volt"));
        buffer.push(Point::new(Timestamp(3), 3.0).with_label("temp"));
        buffer.push(Point::new(Timestamp(4), 4.0));

        let temps: Vec<_> = buffer.iter_by_name("temp").collect();
        assert_eq!(temps.len(), 2);

        let filtered = buffer.filtered_by_name("volt");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.last().unwrap().value(), 2.0);
    }
}
