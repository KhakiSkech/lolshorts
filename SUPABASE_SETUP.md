# Supabase Local Development Setup

This guide will help you set up the local Supabase environment with Google OAuth authentication.

## Prerequisites

- Docker and Docker Compose installed
- Google Cloud Console account for OAuth credentials

## Step 1: Configure Google OAuth

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to **APIs & Services** → **Credentials**
4. Click **Create Credentials** → **OAuth 2.0 Client ID**
5. Configure OAuth consent screen if prompted
6. Select **Web application** as the application type
7. Add these authorized redirect URIs:
   - `http://localhost:8000/auth/v1/callback` (Supabase Auth callback)
   - `http://localhost:1420` (Tauri deep link)
8. Copy the **Client ID** and **Client Secret**

## Step 2: Configure Environment Variables

1. Open `.env.local` in the project root
2. Replace the Google OAuth placeholders:

```bash
# Replace these with your actual Google OAuth credentials
GOOGLE_CLIENT_ID=your-actual-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-actual-client-secret
```

## Step 3: Start Supabase Services

```bash
# Start all Supabase services in detached mode
docker-compose up -d

# Check if all services are running
docker-compose ps

# View logs if needed
docker-compose logs -f auth
```

Expected services:
- ✅ **postgres** - Database (Port 5432)
- ✅ **studio** - Database UI (Port 3001)
- ✅ **kong** - API Gateway (Port 8000)
- ✅ **auth** - GoTrue Auth Server (Port 9999)
- ✅ **rest** - PostgREST API (Port 3000)
- ✅ **realtime** - Realtime subscriptions (Port 4000)
- ✅ **storage** - File storage (Port 5000)
- ✅ **meta** - Database metadata (Port 8080)
- ✅ **inbucket** - Email testing (Port 9000)

## Step 4: Apply Database Migrations

The migration will automatically run when the database starts. To verify:

1. Access Supabase Studio at http://localhost:3001
2. Navigate to **Table Editor**
3. Verify these tables exist:
   - `user_profiles`
   - `games`
   - `clips`
   - `auto_edit_results`
   - `youtube_uploads`
   - `quota_usage`

## Step 5: Test Authentication

### Test Email/Password Signup

1. Run the app: `pnpm run tauri:dev`
2. Click "Login / Sign Up" in the sidebar
3. Switch to "Sign Up" tab
4. Enter email and password
5. Click "Sign Up"
6. Check console for any errors

### Test Google OAuth

1. Click "Sign up with Google" button
2. You should be redirected to Google login
3. After successful login, you'll be redirected back to the app
4. Check if user profile is created in `user_profiles` table

## Accessing Supabase Services

### Supabase Studio (Database UI)
- URL: http://localhost:3001
- Use this to view and edit database tables

### Inbucket (Email Testing)
- URL: http://localhost:9000
- View email verification messages sent during signup

### Kong API Gateway
- URL: http://localhost:8000
- All API requests go through this gateway

## Troubleshooting

### Services Not Starting

```bash
# Check Docker logs
docker-compose logs

# Restart specific service
docker-compose restart auth

# Rebuild and restart
docker-compose down
docker-compose up -d --build
```

### Database Connection Issues

```bash
# Connect to postgres directly
docker-compose exec postgres psql -U postgres

# List databases
\l

# Connect to postgres database
\c postgres

# List tables
\dt public.*
```

### OAuth Redirect Not Working

1. Verify redirect URIs in Google Cloud Console match exactly:
   - `http://localhost:8000/auth/v1/callback`
   - `http://localhost:1420`
2. Check `.env.local` has correct Client ID and Secret
3. Restart auth service: `docker-compose restart auth`

### Profile Not Created After OAuth Login

Check browser console for errors. The profile should be auto-created in `App.tsx` when the auth state changes.

## Database Schema

### user_profiles
- `id` (UUID) - Primary key, references `auth.users`
- `email` (TEXT) - User email
- `display_name` (TEXT) - Optional display name
- `avatar_url` (TEXT) - Optional avatar URL
- `tier` (TEXT) - "FREE" or "PRO"
- `subscription_status` (TEXT) - active, canceled, expired, trialing
- `subscription_expires_at` (TIMESTAMPTZ) - Subscription expiry date
- `created_at` (TIMESTAMPTZ)
- `updated_at` (TIMESTAMPTZ)

All tables have Row Level Security (RLS) enabled, ensuring users can only access their own data.

## Stopping Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (WARNING: Deletes all data)
docker-compose down -v
```

## Exporting Database for Production

Once development is complete, export the schema:

```bash
# Export schema only (no data)
docker-compose exec postgres pg_dump -U postgres -s postgres > schema.sql

# Export schema and data
docker-compose exec postgres pg_dump -U postgres postgres > full_backup.sql
```

Apply to production Supabase:
1. Create a Supabase project at https://app.supabase.com
2. Go to SQL Editor
3. Paste and execute the exported SQL
4. Update `.env.local` with production credentials

## Default Credentials

**Database:**
- Username: `postgres`
- Password: `postgres`
- Database: `postgres`
- Port: `5432`

**JWT Secret:**
- Secret: `your-super-secret-jwt-token-with-at-least-32-characters-long`

**Anon Key (for client):**
- Already configured in `src/lib/supabase.ts`

⚠️ **Important:** Change these credentials before deploying to production!
