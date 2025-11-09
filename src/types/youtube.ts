// YouTube integration type definitions

export interface YouTubeVideo {
  video_id: string;
  title: string;
  description: string;
  published_at: string;
  thumbnail_url: string;
  view_count: number;
  like_count: number;
  comment_count: number;
  privacy_status: PrivacyStatus;
  url: string;
}

export type PrivacyStatus = 'Public' | 'Unlisted' | 'Private';

export interface VideoMetadata {
  title: string;
  description: string;
  tags: string[];
  privacy_status: PrivacyStatus;
  made_for_kids: boolean;
  category_id: string;
}

export interface UploadProgress {
  uploaded_bytes: number;
  total_bytes: number;
  status: UploadStatus;
  video_id: string | null;
}

export type UploadStatus =
  | 'Pending'
  | 'Uploading'
  | 'Processing'
  | 'Completed'
  | 'Failed';

export interface UploadHistoryEntry {
  video: YouTubeVideo;
  uploaded_at: number; // Unix timestamp
  local_file_path: string;
}

export interface QuotaInfo {
  daily_limit: number;
  used: number;
  remaining: number;
  reset_at: number; // Unix timestamp
}

export interface AuthStatus {
  authenticated: boolean;
  email: string | null;
  expires_at: number | null; // Unix timestamp
}
