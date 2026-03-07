use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Draft,
    Hidden,
    Scheduled,
    Onboarding,
    Onboarded,
    Pending,
    Failed,
    Training,
    Rejected,
    Enabling,
    Deleting,
    Disabled,
    Deleted,
    InProgress,
    Completed,
    Canceling,
    Canceled,
    DeprecatedDraft,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Onboarded => write!(f, "Online"),
            Self::Draft => write!(f, "Draft"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Unknown => write!(f, "Unknown"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileType {
    Audio,
    Video,
    Image,
    Text,
    Application,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenType {
    Input,
    Output,
    Total,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ownership {
    Private,
    Public,
    Team,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Function {
    #[serde(rename = "text-generation")]
    TextGeneration,
    #[serde(rename = "translation")]
    Translation,
    #[serde(rename = "speech-recognition")]
    SpeechRecognition,
    #[serde(rename = "text-to-speech")]
    TextToSpeech,
    #[serde(rename = "text-summarization")]
    TextSummarization,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "classification")]
    Classification,
    #[serde(rename = "text-to-image")]
    TextToImage,
    #[serde(rename = "speech-enhancement")]
    SpeechEnhancement,
    #[serde(rename = "sentiment-analysis")]
    SentimentAnalysis,
    #[serde(rename = "question-answering")]
    QuestionAnswering,
    #[serde(rename = "image-classification")]
    ImageClassification,
    #[serde(rename = "object-detection")]
    ObjectDetection,
    #[serde(rename = "utilities")]
    Utilities,
    #[serde(other)]
    Other,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextGeneration => write!(f, "Text Generation"),
            Self::Translation => write!(f, "Translation"),
            Self::SpeechRecognition => write!(f, "Speech Recognition"),
            Self::TextToSpeech => write!(f, "Text to Speech"),
            Self::TextSummarization => write!(f, "Text Summarization"),
            Self::Search => write!(f, "Search"),
            Self::Classification => write!(f, "Classification"),
            Self::TextToImage => write!(f, "Text to Image"),
            Self::SpeechEnhancement => write!(f, "Speech Enhancement"),
            Self::SentimentAnalysis => write!(f, "Sentiment Analysis"),
            Self::QuestionAnswering => write!(f, "Question Answering"),
            Self::ImageClassification => write!(f, "Image Classification"),
            Self::ObjectDetection => write!(f, "Object Detection"),
            Self::Utilities => write!(f, "Utilities"),
            Self::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_status_deserialize() {
        let s: AssetStatus = serde_json::from_str(r#""onboarded""#).unwrap();
        assert_eq!(s, AssetStatus::Onboarded);

        let s: AssetStatus = serde_json::from_str(r#""draft""#).unwrap();
        assert_eq!(s, AssetStatus::Draft);

        let s: AssetStatus = serde_json::from_str(r#""in_progress""#).unwrap();
        assert_eq!(s, AssetStatus::InProgress);

        let s: AssetStatus = serde_json::from_str(r#""some_future_value""#).unwrap();
        assert_eq!(s, AssetStatus::Unknown);
    }

    #[test]
    fn function_deserialize() {
        let f: Function = serde_json::from_str(r#""text-generation""#).unwrap();
        assert_eq!(f, Function::TextGeneration);
        assert_eq!(f.to_string(), "Text Generation");

        let f: Function = serde_json::from_str(r#""unknown-func""#).unwrap();
        assert_eq!(f, Function::Other);
    }

    #[test]
    fn ownership_round_trip() {
        let o = Ownership::Private;
        let json = serde_json::to_string(&o).unwrap();
        assert_eq!(json, r#""PRIVATE""#);
        let back: Ownership = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Ownership::Private);
    }
}
