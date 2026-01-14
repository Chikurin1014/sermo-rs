use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Message delivered to UI representing bytes received/sent on a port
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    pub direction: Direction,
    /// Message payload as a UTF-8 string (display-friendly)
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
}

/// A single data point with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// Timestamp in milliseconds since UNIX_EPOCH
    pub timestamp: u64,
    /// Optional name or label for the data point
    pub label: Option<String>,
    /// The data value (e.g., sensor reading)
    pub value: f64,
}

impl Point {
    pub fn new(value: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            timestamp,
            label: None,
            value,
        }
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
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

    /// Add a value with current timestamp
    pub fn push_value(&mut self, value: f64) {
        self.push(Point::new(value));
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
        let point = Point::new(42.0);
        assert_eq!(point.value, 42.0);
    }

    #[test]
    fn test_data_buffer_push() {
        let mut buffer = PointBuffer::new(10);
        buffer.push_value(1.0);
        buffer.push_value(2.0);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_data_buffer_last() {
        let mut buffer = PointBuffer::new(10);
        buffer.push_value(1.0);
        buffer.push_value(2.0);
        assert_eq!(buffer.last().unwrap().value, 2.0);
    }

    #[test]
    fn test_data_buffer_filter_by_name() {
        let mut buffer = PointBuffer::new(10);
        buffer.push(Point::new(1.0).with_label("temp"));
        buffer.push(Point::new(2.0).with_label("volt"));
        buffer.push(Point::new(3.0).with_label("temp"));
        buffer.push(Point::new(4.0));

        let temps: Vec<_> = buffer.iter_by_name("temp").collect();
        assert_eq!(temps.len(), 2);

        let filtered = buffer.filtered_by_name("volt");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.last().unwrap().value, 2.0);
    }
}
