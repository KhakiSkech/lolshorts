# Riot Games Compliance Document

**Application Name**: LoLShorts
**Last Updated**: January 4, 2025
**Purpose**: Document compliance with Riot Games policies and legal requirements

## 1. Legal Relationship with Riot Games

### 1.1 Independent Third-Party Application
LoLShorts is an **independent third-party application** and is:
- ❌ NOT endorsed by Riot Games, Inc.
- ❌ NOT sponsored by Riot Games, Inc.
- ❌ NOT affiliated with Riot Games, Inc.
- ✅ A community-created tool for League of Legends players

### 1.2 Trademark Acknowledgment
- "League of Legends" is a trademark of Riot Games, Inc.
- "Riot Games" is a trademark of Riot Games, Inc.
- All League of Legends assets, names, and logos are property of Riot Games
- Our use of these names is for **identification purposes only**

### 1.3 Required Disclaimer

All public-facing materials include:

> LoLShorts was created under Riot Games' "Legal Jibber Jabber" policy using assets owned by Riot Games. Riot Games does not endorse or sponsor this project.

## 2. Riot Games Policies Compliance

### 2.1 Terms of Service Compliance
LoLShorts adheres to Riot Games' Terms of Service:
- **No gameplay automation**: Does not automate gameplay actions
- **No unfair advantage**: Does not provide competitive advantages
- **No unauthorized access**: Uses only official APIs and publicly available data
- **No account security compromise**: Does not require League account credentials
- **User responsibility**: Users must comply with Riot TOS when playing League

**Riot TOS**: https://www.riotgames.com/en/terms-of-service

### 2.2 Third-Party Application Policy
As per Riot's Third-Party Application Policy, LoLShorts:

✅ **DOES**:
- Use only official, public-facing APIs
- Access League Client Update (LCU) API (officially supported)
- Access Live Client Data API (officially supported)
- Record gameplay video (Windows Game DVR)
- Process replay files

❌ **DOES NOT**:
- Modify game files
- Inject code into the game client
- Read game memory
- Automate gameplay actions
- Provide scripting capabilities
- Access unauthorized game data
- Bypass Riot's security measures

**Riot Developer Portal**: https://developer.riotgames.com/

### 2.3 Content Usage Policy
LoLShorts complies with Riot's "Legal Jibber Jabber" policy for content usage:

✅ **Permitted**:
- Recording gameplay footage
- Creating highlight videos
- Sharing videos on social media
- Using game assets for thumbnails (from official APIs)
- Monetizing created content (per Riot's Creator Policy)

❌ **Prohibited**:
- Extracting and distributing Riot's game assets
- Creating competing products using Riot's IP
- Implying official endorsement by Riot
- Using Riot's logos in a misleading way

**Legal Jibber Jabber**: https://www.riotgames.com/en/legal-jibber-jabber

## 3. API Usage Compliance

### 3.1 League Client Update (LCU) API
- **Purpose**: Detect game state and summoner information
- **Access Method**: WebSocket connection to `https://127.0.0.1:2999`
- **Data Collected**:
  - Summoner name (public information)
  - Current game status (in-game, lobby, etc.)
  - Champion selection
  - Game mode
- **Compliance**: Local API, no rate limits, officially supported

### 3.2 Live Client Data API
- **Purpose**: Detect in-game events for clip creation
- **Access Method**: HTTPS requests to `https://127.0.0.1:2999/liveclientdata/`
- **Data Collected**:
  - Game events (kills, objectives)
  - Game time
  - Player names (in current match)
- **Compliance**: Local API, officially supported for spectator features

### 3.3 API Rate Limiting
- All APIs used are **local-only** (no remote Riot servers accessed)
- No rate limit concerns as data is retrieved from user's own game client
- No API keys required for local-only access

### 3.4 Data Retention
- No permanent storage of Riot's game data
- Metadata stored locally (timestamps, event types)
- No redistribution of Riot's proprietary data

## 4. Content Creation Guidelines

### 4.1 User-Generated Content
LoLShorts enables users to create content. Users must:
- ✅ Own the gameplay footage they record (their own matches)
- ✅ Comply with Riot's content creation policies
- ✅ Not misrepresent their relationship with Riot
- ❌ Not create content that violates Riot's community standards

### 4.2 Monetization
Per Riot's Creator Policy:
- ✅ Users MAY monetize gameplay videos (YouTube, Twitch, etc.)
- ✅ Users MAY use League assets in thumbnails
- ❌ Users may NOT claim ownership of League of Legends
- ❌ Users may NOT imply official Riot endorsement

**Creator Policy**: https://www.riotgames.com/en/creator-policy

## 5. Brand Usage Guidelines

### 5.1 Application Naming
- "LoLShorts" is distinct from Riot's official products
- Does not use "Riot", "League", or "LoL" as primary branding
- "LoL" is used as prefix for product category identification only

### 5.2 Visual Identity
- Does not use Riot's official logos
- Does not mimic Riot's brand colors or design language
- Uses distinct branding and visual style

### 5.3 Marketing Language
All promotional materials:
- ✅ Include disclaimer about independent third-party status
- ✅ State "for League of Legends" not "by Riot Games"
- ❌ Do NOT imply partnership, sponsorship, or endorsement

**Example Compliant Statement**:
> "LoLShorts is a third-party application for creating League of Legends highlight videos. Not affiliated with Riot Games."

## 6. User Privacy and Security

### 6.1 Account Security
- LoLShorts does NOT require League of Legends login credentials
- Users authenticate with LoLShorts using separate credentials
- No storage of Riot account passwords
- No access to user's Riot account beyond public game data

### 6.2 Data Protection
- Gameplay recordings stored locally on user's device
- No unauthorized transmission of game data to external servers
- Compliance with GDPR/CCPA for user data (see Privacy Policy)

## 7. Anti-Cheat Compliance

### 7.1 Vanguard Compatibility
LoLShorts is designed to work alongside Riot's Vanguard anti-cheat:
- No kernel-level operations
- No game memory access
- No code injection
- No process manipulation
- Uses only legitimate Windows and Riot APIs

### 7.2 Detection Risk
LoLShorts is designed to minimize any false-positive risk:
- No interaction with game process
- No suspicious system calls
- Operates as standard desktop application

**If Flagged**: Users should contact Riot Support and reference this compliance document

## 8. Community Standards

### 8.1 Encouraging Positive Play
LoLShorts aligns with Riot's goal of positive community experiences:
- Enables sharing of highlight moments
- Encourages celebrating achievements
- No features that enable toxicity or harassment

### 8.2 Prohibited Content
LoLShorts does not enable creation of:
- ❌ Toxic or harassment content
- ❌ Cheat showcases or exploit demonstrations
- ❌ Content that violates Riot's Community Code

## 9. Intellectual Property Respect

### 9.1 Copyright Compliance
- Users record their own gameplay (fair use)
- No redistribution of Riot's copyrighted material
- Compliance with DMCA for user-generated content

### 9.2 Trademark Compliance
- Use of "League of Legends" trademark is nominative fair use
- No registration of confusingly similar trademarks
- Respect for Riot's trademark rights

## 10. Riot's Right to Request Changes

### 10.1 Compliance Commitment
We commit to:
- Promptly respond to Riot Games' requests or concerns
- Modify or remove features if requested by Riot
- Discontinue the application if Riot determines it violates policies

### 10.2 Communication Channel
Riot Games can contact us at:
- **Email**: legal@lolshorts.com
- **Response Time**: Within 48 hours for urgent requests

## 11. Updates and Changes

### 11.1 Policy Monitoring
We actively monitor:
- Riot Games Terms of Service updates
- Developer Portal policy changes
- Community guidelines updates

### 11.2 Proactive Compliance
- Regular review of Riot's policies (quarterly)
- Immediate updates if policies change
- User notification of compliance-related changes

## 12. Transparency Report

### 12.1 API Usage Statistics
- Local LCU API: ✅ Used for game detection
- Local Live Client Data API: ✅ Used for event detection
- Remote Riot APIs: ❌ Not used
- Riot Production Keys: ❌ Not applicable (local-only)

### 12.2 Compliance Incidents
As of January 4, 2025:
- ✅ No violations reported
- ✅ No cease-and-desist notices received
- ✅ No policy conflicts identified

## 13. User Education

### 13.1 In-App Notices
LoLShorts includes:
- Disclaimer about third-party status
- Link to Riot's Terms of Service
- Reminders about complying with Riot's policies

### 13.2 Documentation
- Terms of Service references Riot's TOS
- User guide explains proper usage
- FAQ addresses common compliance questions

## 14. Enforcement and Termination

### 14.1 User Violations
If users violate Riot's policies while using LoLShorts:
- User is solely responsible
- LoLShorts may terminate user's account
- User may face penalties from Riot Games

### 14.2 Application Termination
If Riot Games requests discontinuation:
- We will comply immediately
- Users will be notified
- Refunds processed as appropriate

## 15. Legal Contact Information

### 15.1 For Riot Games
To contact LoLShorts regarding compliance:

**Legal Department**
- **Email**: legal@lolshorts.com
- **Priority Response**: Riot Games inquiries prioritized
- **Response Time**: Within 48 hours

### 15.2 For Users
Questions about Riot compliance:
- **Email**: support@lolshorts.com
- **FAQ**: https://lolshorts.com/faq#riot-compliance

## 16. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-04 | Initial compliance document |

## 17. Attestation

LoLShorts certifies that:
- ✅ We have reviewed Riot Games' policies
- ✅ We are committed to full compliance
- ✅ We will cooperate with Riot Games' requests
- ✅ We respect Riot's intellectual property rights
- ✅ We do not claim any affiliation with Riot Games

---

**Document Owner**: LoLShorts Legal Team
**Review Frequency**: Quarterly
**Next Review Date**: April 4, 2025

**For Riot Games Legal Team**: If you have any questions or concerns about this application, please contact us immediately at legal@lolshorts.com. We are committed to working cooperatively with Riot Games.
