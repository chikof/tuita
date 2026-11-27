app_name = tuita
loading = Loading
close = Close
dismiss = Dismiss
notifications = Notifications
avatar_more = { $count } more
char_counter_left =
    { $count ->
        [one] { $count } character left
       *[other] { $count } characters left
    }
char_counter_over =
    { $count ->
        [one] { $count } character over the limit
       *[other] { $count } characters over the limit
    }
not_found_title = Nothing here
not_found_body = This page does not exist, or it moved.
not_found_back = Go back
tagline = Every post, two inks.
error_title = Something went wrong
cancel = Cancel
retry = Try again
back = Back
view = View
follow = Follow
following = Following
follows_you = Follows you
verified = Verified account
bot_account = Automated account
bot_label = Bot
link_copied = Link copied to clipboard
text_copied = Text copied
time_now = now
time_minutes = { $n }m
time_hours = { $n }h
time_days = { $n }d
nav_main = Main
nav_home = Home
nav_explore = Explore
nav_notifications = Notifications
nav_bookmarks = Bookmarks
nav_profile = Profile
nav_settings = Settings
nav_compose = Post
nav_menu = Menu
badge_unread = { $count } unread
account_menu = Accounts
account_switch = Switch account
account_switch_to = Switch to { $handle }
account_add = Add an account
account_logout = Log out { $handle }
account_logout_all = Log out of all accounts
home_feed = Timeline
home_for_you = For you
home_following = Following
home_empty_title = Nothing here yet
home_empty_body = Be the first to post something.
home_following_empty_title = Your circle is quiet
home_following_empty_body = Follow people to fill this timeline with their posts and reposts.
home_find_people = Find people
new_posts =
    { $count ->
        [one] Show { $count } new post
       *[other] Show { $count } new posts
    }
new_posts_many = Show 99+ new posts
composer_title = New post
composer_reply_title = Reply
composer_quote_title = Quote post
composer_placeholder = What’s happening?
composer_reply_placeholder = Post your reply
composer_text_label = Post text
composer_replying_to = Replying to { $handle }
composer_cw_label = Content warning
composer_cw_placeholder = Content warning, e.g. spoilers
composer_cw_toggle = Add a content warning
composer_sensitive_toggle = Mark media as sensitive
composer_options = Post options
composer_policy_label = Who can reply
composer_suggestions = Suggestions
composer_tag_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
composer_post = Post
composer_reply = Reply
policy_everyone = Everyone can reply
policy_following = People you follow can reply
policy_mentioned = Only people you mention can reply
post_sent = Post sent
post_by = Post by { $name }
post_reposted_by = { $name } reposted
post_pinned = Pinned post
post_replying_to = Replying to { $handle }
post_replying_unavailable = Replying to a post that is unavailable
post_unavailable = This post is unavailable.
post_menu = More
post_copy_link = Copy link
post_copy_text = Copy text
post_pin = Pin to profile
post_unpin = Unpin from profile
post_pinned_toast = Pinned to your profile
post_unpinned = Unpinned from your profile
post_delete = Delete post
post_delete_title = Delete post?
post_delete_body = This can’t be undone. It is removed from your profile, from the timelines of people who follow you and from search.
post_deleted = Post deleted
post_view_profile = View { $handle }
post_mute = Mute { $handle }
post_block = Block { $handle }
post_reply_label = Reply ({ $count })
post_reply_following_only = Only people the author follows can reply.
post_reply_mentioned_only = Only people mentioned in this post can reply.
post_repost_menu = Repost options
post_repost = Repost
post_undo_repost = Undo repost
post_quote = Quote
post_reposts_label = Reposts and quotes ({ $count })
post_like_label = Like ({ $count })
post_bookmark_label = Bookmark
post_share = Share or copy link
cw_show = Show
cw_hide = Hide
cw_label = Content warning: { $reason }
muted_toast = Muted { $handle }
unmuted_toast = Unmuted { $handle }
blocked_toast = Blocked { $handle }
unblocked_toast = Unblocked { $handle }
block_title = Block { $handle }?
block_body = They can’t follow you and you won’t see their posts. They are not told.
block_confirm = Block
notify_like = @{ $handle } liked your post
notify_repost = @{ $handle } reposted your post
notify_reply = @{ $handle } replied to you
notify_mention = @{ $handle } mentioned you
notify_quote = @{ $handle } quoted your post
notify_follow = @{ $handle } followed you
notify_follow_request = @{ $handle } asked to follow you
notify_poll = A poll you voted in has ended
notify_affiliation = @{ $handle } invited you to be affiliated
shortcuts_title = Keyboard shortcuts
shortcut_group_navigation = Navigation
shortcut_group_actions = Actions
shortcut_or = /
shortcut_then = then
shortcut_plus = +
shortcut_next_prev = Next / previous post
shortcut_open = Open post
shortcut_search = Search
shortcut_go_home = Home
shortcut_go_explore = Explore
shortcut_go_notifications = Notifications
shortcut_go_bookmarks = Bookmarks
shortcut_go_profile = Your profile
shortcut_go_settings = Settings
shortcut_new_post = New post
shortcut_like = Like
shortcut_reply = Reply
shortcut_repost = Repost or quote
shortcut_bookmark = Bookmark
shortcut_send = Send post
shortcut_help = Show this help
auth_login_title = Sign in
auth_add_title = Add another account
auth_add_sub = Sign in to switch between up to { $max } accounts on this device without signing out.
auth_signup_title = Create account
auth_handle = Handle
auth_password = Password
auth_display_name = Display name
auth_handle_hint = 3–15 letters, numbers or underscores
auth_password_hint = At least 8 characters
auth_consent_pre = I’m at least { $minAge } and I agree to the{" "}
auth_consent_terms = Terms of Service
auth_consent_and = {" "}and{" "}
auth_consent_privacy = Privacy Policy
auth_consent_end = {"."}
auth_consent_required = Please accept the Terms of Service and Privacy Policy.
auth_signin = Sign in
auth_signin_add = Sign in and switch
auth_create = Create account
auth_to_signup = New here? Create an account
auth_to_login = Have an account? Sign in
auth_closed = Sign-ups are closed on this instance right now.
landing_nav = Footer
landing_rules = House rules
landing_stats = Right now
landing_people = People
landing_posts = Posts
landing_clock = Your local time
landing_trends = Trending
landing_trend =
    { $count ->
        [one] { $count } post
       *[other] { $count } posts
    }
landing_join = Join { $name }
landing_join_sub = Make an account and post in two inks.
landing_closed = Sign-ups are closed right now. Already have an account? Sign in.
landing_terms = Terms
landing_privacy = Privacy
landing_source = Source
landing_powered = Powered by tuita { $version }
docs_terms_title = Terms of Service
docs_privacy_title = Privacy Policy
docs_updated = Last updated { $date }
docs_date = 5 November 2026
docs_see_privacy = See also the Privacy Policy.
docs_see_terms = See also the Terms of Service.
docs_contact_fallback = the instance administrators (via a direct message or a report)
docs_terms_agreement_h = 1. Agreement
docs_terms_agreement_p = These Terms of Service (“Terms”) govern your use of { $name }, the tuita instance at { $domain } (“the service”, “we”). By creating an account or using the service you agree to these Terms and to our Privacy Policy. If you don’t agree, don’t use the service.
docs_terms_eligibility_h = 2. Who can use it
docs_terms_eligibility_p = You must be at least { $minAge } years old, or the minimum age of digital consent where you live if that is higher. You must be able to form a binding agreement and not be barred from using the service under applicable law.
docs_terms_account_h = 3. Your account
docs_terms_account_1 = Keep your password secret. You’re responsible for activity on your account.
docs_terms_account_2 = Your handle must not impersonate another person or organisation or mislead people about who you are.
docs_terms_account_3 = Tell us promptly if you think your account has been compromised. You can sign out other sessions from Settings → Account.
docs_terms_content_h = 4. Your content
docs_terms_content_p1 = You keep ownership of what you post. You give us a worldwide, non-exclusive, royalty-free licence to store, display, reproduce and distribute your content only to operate and improve the service. That includes sending it to federated tuita instances (see section 6). This licence ends when you delete the content or your account, except for copies already delivered to other instances or saved by other people.
docs_terms_content_p2 = You’re responsible for what you post. Only post content you have the right to share.
docs_terms_rules_h = 5. Community rules
docs_terms_rules_lead = When using { $name } you agree not to break the law or these rules:
docs_terms_rules_p = You also agree not to attack the service, for example by trying to get around security measures, scraping it at scale, sending malware or overloading it.
docs_terms_federation_h = 6. Federation
docs_terms_federation_p = { $name } can connect (“federate”) with other independent tuita instances that the administrators approve. When you interact with people on another instance, your public profile, public posts and the interaction (for example a follow, like, repost, reply or direct message) are sent to that instance. Other instances are run by other people under their own terms. We can’t control or delete content once another instance has it, though we do send them deletion requests when you delete something.
docs_terms_moderation_h = 7. Moderation and enforcement
docs_terms_moderation_p = Moderators may remove content, limit features or suspend accounts that break these Terms or the law. Where possible we will tell you why. Anyone can report a post or account. Reports go to this instance’s moderators. We may stop federating with any instance at any time.
docs_terms_termination_h = 8. Leaving
docs_terms_termination_p = You can export your data or delete your account at any time from Settings → Account. We may suspend or end your access if you seriously or repeatedly break these Terms. We may also close the service, and will try to give reasonable notice first.
docs_terms_disclaimers_h = 9. Disclaimers and liability
docs_terms_disclaimers_p = The service is provided “as is” and “as available”, without warranties of any kind, to the fullest extent the law allows. We are not liable for indirect, incidental or consequential damages, or for content posted by users or other instances. Nothing in these Terms limits liability that cannot be limited by law, or your statutory rights as a consumer.
docs_terms_changes_h = 10. Changes
docs_terms_changes_p = We may update these Terms. If a change is significant we will give notice in the app before it takes effect. Continuing to use the service after that means you accept the new Terms.
docs_terms_contact_h = 11. Contact
docs_terms_contact_lead = Questions about these Terms:
docs_privacy_summary_h = The short version
docs_privacy_summary_1 = No ads, no tracking scripts, no selling of data.
docs_privacy_summary_2 = One cookie, to keep you signed in. Theme preferences stay in your browser.
docs_privacy_summary_3 = Public posts are public, and can reach federated tuita instances.
docs_privacy_summary_4 = Direct messages between accounts on this server are end-to-end encrypted once both people turn it on: they are stored as text the server can’t read, so a database leak or a look at the stored data reveals nothing. Messages to people on other servers, and messages sent before you turned encryption on, are not encrypted this way and administrators can technically read them. This server also delivers the app that does the encrypting, so an administrator who changed that app could capture what you type.
docs_privacy_summary_5 = You can export or delete your data whenever you want.
docs_privacy_controller_h = 1. Who we are
docs_privacy_controller_p = This policy covers { $name } at { $domain }, run by its instance administrators. They decide how your data is processed.
docs_privacy_collect_h = 2. What we collect
docs_privacy_collect_1 = Account data: handle, display name, password (stored only as a salted argon2id hash), and your profile details: bio, location, website, avatar, banner, shape and colour.
docs_privacy_collect_2 = Content: posts, images, replies, likes, reposts, bookmarks, follows, blocks, mutes and reports.
docs_privacy_collect_3 = Direct messages: stored on this server so they can be delivered to you and the other person. An end-to-end encrypted message is stored as ciphertext that we cannot read; who sent it, to whom and when stay visible to us. Other messages are stored as plain text. Your encryption key is stored here only in a form locked by your passphrase, which we never see.
docs_privacy_collect_4 = Session data: for each signed-in device, the browser’s user-agent, IP address and last-used time, so you can review and sign out sessions.
docs_privacy_collect_5 = Short-lived security data: IP addresses kept in memory for a few minutes for rate limiting.
docs_privacy_collect_p = We don’t collect your email, phone number, contacts or location unless you type them into your profile.
docs_privacy_use_h = 3. How we use it
docs_privacy_use_p = Only to run the service: show your profile and posts, build timelines and notifications, deliver messages, keep accounts secure, prevent abuse and enforce the Terms. The “For you” timeline ranks posts by likes, reposts, replies and age. It doesn’t profile you.
docs_privacy_visibility_h = 4. What other people can see
docs_privacy_visibility_p = Your profile, posts, replies, reposts, likes, followers and following are public to anyone signed in to this instance and to federated instances. When someone shares a link to a post or profile, apps like Discord, Slack or iMessage can fetch a preview showing the public post, its images and your public profile details. Posts behind a content warning or marked sensitive are previewed without their text and images. Bookmarks, blocks, mutes, reports and settings are private. If you turn off “Show me in suggestions”, people can still find you by your exact handle.
docs_privacy_federation_h = 5. Federation
docs_privacy_federation_p = When you follow, reply to, mention, like, repost or message someone on another tuita instance, we send the needed data to that instance: your public profile, the post or interaction and, for direct messages, the message text. Direct messages to other instances are not end-to-end encrypted. Your followers’ instances receive your public posts and profile updates. Those instances are run independently and store data under their own policies. When you delete a post or your account we ask federated instances to delete their copies, but we can’t guarantee that they do.
docs_privacy_cookies_h = 6. Cookies and local storage
docs_privacy_cookies_p = We use one strictly necessary, HttpOnly session cookie. Your theme, colour, text size and motion preferences are stored in your browser’s local storage and never sent to us.
docs_privacy_retention_h = 7. How long we keep data
docs_privacy_retention_1 = Content and account data are kept until you delete them.
docs_privacy_retention_2 = Deleted posts are removed from public view immediately, and their text and images are cleared.
docs_privacy_retention_3 = When you delete your account we remove your posts, likes, follows, messages and sessions, and anonymise the account record.
docs_privacy_retention_4 = Sessions expire after 30 days, or straight away when you sign out.
docs_privacy_retention_5 = Moderation reports and the admin audit log are kept as long as needed to keep the service safe.
docs_privacy_rights_h = 8. Your rights
docs_privacy_rights_p = Depending on where you live (for example under the GDPR or CCPA) you may have the right to access, correct, export or delete your data, and to object to or restrict processing. You can do most of this yourself in Settings → Account: edit your profile, download your data and delete your account. For anything else, contact:
docs_privacy_rights_end = You may also complain to your local data protection authority.
docs_privacy_security_h = 9. Security
docs_privacy_security_p = Passwords are hashed with scrypt. Session tokens are stored only as hashes. Requests between federated instances are signed with Ed25519 keys. No system is perfectly secure, so please use a unique password.
docs_privacy_children_h = 10. Children
docs_privacy_children_p = The service isn’t meant for children under the minimum age in the Terms. If you think a child has an account, report it.
docs_privacy_changes_h = 11. Changes
docs_privacy_changes_p = We’ll post changes to this policy here and update the date above. Significant changes will be announced in the app.
docs_privacy_contact_h = 12. Contact
docs_privacy_contact_lead = Privacy questions or requests:
profile_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
profile_edit = Edit profile
profile_more = More
profile_copy_link = Copy profile link
profile_mute = Mute { $handle }
profile_unmute = Unmute { $handle }
profile_block = Block { $handle }
profile_unblock = Unblock { $handle }
profile_unblock_button = Unblock
profile_suspended = This account is suspended. Only staff can see it.
profile_muted = Muted
profile_org = Organization
profile_role_admin = Admin
profile_role_moderator = Mod
profile_joined = Joined { $date }
profile_following_label = Following
profile_followers =
    { $count ->
        [one] Follower
       *[other] Followers
    }
profile_counts = Follow counts
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Followed by { $a }
            }
        [2]
            { $others ->
               *[other] Followed by { $a } and { $b }
            }
       *[other]
            { $others ->
                [one] Followed by { $a }, { $b } and { $others } other you follow
               *[other] Followed by { $a }, { $b } and { $others } others you follow
            }
    }
profile_banner = Profile banner
profile_tabs = Profile sections
profile_tab_posts = Posts
profile_tab_replies = Replies
profile_tab_media = Media
profile_tab_likes = Likes
profile_posts_label = Posts
profile_empty_posts_self_title = You haven’t posted yet
profile_empty_posts_self_body = Your posts and reposts will show up here.
profile_empty_posts_title = { $handle } hasn’t posted
profile_empty_posts_body = When they do, their posts will show up here.
profile_empty_replies_title = No replies yet
profile_empty_replies_body = Replies will show up here.
profile_empty_media_title = No photos yet
profile_empty_media_body = Posts with images will show up here.
profile_empty_likes_title = No likes yet
profile_empty_likes_body = Liked posts will show up here.
profile_blocked_title = You blocked { $handle }
profile_blocked_body = You won’t see their posts, and they can’t follow or message you.
profile_hidden_title = You can’t see these posts
profile_hidden_body = { $handle } has limited who can see their profile.
follows_tabs = Follow lists
follows_tab_followers = Followers
follows_tab_following = Following
follows_label_followers = Followers
follows_label_following = Following
follows_empty_followers_title = No followers yet
follows_empty_followers_body = When people follow this account, they’ll show up here.
follows_empty_following_title = Not following anyone
follows_empty_following_body = Accounts they follow will show up here.
thread_title = Post
thread_doc_title = { $name }: “{ $text }”
thread_views =
    { $count ->
        [one] View
       *[other] Views
    }
thread_reposts =
    { $count ->
        [one] Repost
       *[other] Reposts
    }
thread_quotes =
    { $count ->
        [one] Quote
       *[other] Quotes
    }
thread_likes =
    { $count ->
        [one] Like
       *[other] Likes
    }
thread_stats = Post statistics
thread_replies = Replies
thread_hidden = Hidden replies ({ $count })
thread_hidden_note = The author hid these replies.
thread_list_likes = Liked by
thread_list_reposts = Reposted by
thread_list_quotes = Quotes
thread_list_empty = Nobody yet
tag_empty_title = No posts with { $tag }
tag_empty_body = Be the first to use it.
tag_posts_label = Posts tagged { $tag }
bookmarks_title = Bookmarks
bookmarks_label = Bookmarked posts
bookmarks_empty_title = Save posts for later
bookmarks_empty_body = Tap the bookmark icon on any post to keep it here. Only you can see your bookmarks.
settings_account = Your account
settings_account_desc = Profile and password
settings_profile = Edit profile
settings_profile_desc = Name, bio, avatar shape and ink
settings_privacy = Privacy & safety
settings_privacy_desc = Messages, discoverability, muted words, blocked and muted accounts
settings_notifications = Notifications
settings_notifications_desc = Choose what you get notified about
settings_appearance = Appearance
settings_appearance_desc = Theme, ink, text size, contrast and motion
settings_language = Language
settings_language_desc = Language of the interface
settings_language_note = Saved in this browser, and on your account when you are signed in.
settings_about = About this instance
settings_about_desc = Rules, terms, privacy, version and keyboard shortcuts
settings_logout = Log out
settings_account_profile = Profile
settings_account_joined = Joined { $date }
settings_account_edit = Edit profile
settings_password_title = Change password
settings_password_current = Current password
settings_password_new = New password
settings_password_hint = At least 8 characters. Other devices will be signed out.
settings_password_confirm = Confirm new password
settings_password_submit = Update password
settings_password_mismatch = The new passwords don’t match.
settings_password_updated = Password updated. Other sessions were signed out.
settings_sessions_title = Devices & sessions
settings_sessions_desc = Sign out anything you don’t recognise, then change your password.
settings_device_on = { $browser } on { $os }
settings_device_unknown = Unknown device
settings_session_current = This device
settings_session_last_active = Last active { $time }
settings_session_sign_out = Sign out
settings_session_sign_out_device = Sign out { $device }
settings_session_signed_out = Session signed out
settings_sessions_revoke_all = Sign out of all other devices
settings_sessions_revoke_title = Sign out other devices?
settings_sessions_revoke_body = Every device except this one will be signed out.
settings_sessions_revoke_confirm = Sign out others
settings_sessions_revoked =
    { $count ->
        [one] { $count } session signed out
       *[other] { $count } sessions signed out
    }
settings_profile_preview = Preview
settings_profile_preview_alt = Avatar preview
settings_profile_uploads = Avatar and banner uploads arrive later. Until then your generated avatar and banner are used.
settings_profile_details = Details
settings_profile_name = Display name
settings_profile_bio = Bio
settings_profile_location = Location
settings_profile_website = Website
settings_profile_shape = Avatar shape
settings_profile_ink = Profile ink
settings_profile_save = Save profile
settings_profile_saved = Profile updated
settings_shape_square = Square
settings_shape_circle = Circle
settings_shape_squircle = Squircle
settings_shape_octagon = Octagon
settings_ink_bone = Bone
settings_ink_phosphor = Phosphor
settings_ink_amber = Amber
settings_ink_cyan = Cyan
settings_ink_rose = Rose
settings_dm_title = Direct messages
settings_dm_desc = Who can start a conversation with you
settings_dm_label = Who can message you
settings_dm_everyone = Everyone
settings_dm_following = People I follow
settings_dm_none = No one
settings_dm_saved = Message settings saved
settings_discover_title = Discoverability
settings_discoverable = Show me in suggestions and search
settings_discoverable_hint = People can still find you by your exact handle
settings_show_client = Show which device I post from
settings_show_client_hint = Adds the device under your new posts
settings_words_title = Muted words
settings_words_desc = Hide posts containing these words, phrases or #hashtags from your timelines, search and replies. Your own posts are never hidden.
settings_words_label = Word to mute
settings_words_placeholder = Add a word, phrase or #hashtag
settings_words_add = Mute
settings_words_remove = Unmute { $word }
settings_words_none = No muted words.
settings_words_limit = You can mute up to { $count } words.
settings_blocked_title = Blocked accounts
settings_blocked_desc = They can’t follow you, message you, or see or interact with your posts. They aren’t told.
settings_blocked_empty = You haven’t blocked anyone.
settings_muted_title = Muted accounts
settings_muted_desc = You won’t see their posts or notifications. They can still follow and message you.
settings_muted_empty = You haven’t muted anyone.
settings_unblock = Unblock
settings_unmute = Unmute
settings_unblock_user = Unblock { $handle }
settings_unmute_user = Unmute { $handle }
settings_notif_title = Notify me about
settings_notif_desc = Muted and blocked accounts never notify you.
settings_notif_like = Likes
settings_notif_like_hint = When someone likes your post
settings_notif_repost = Reposts
settings_notif_repost_hint = When someone reposts your post
settings_notif_reply = Replies
settings_notif_reply_hint = When someone replies to you
settings_notif_mention = Mentions
settings_notif_mention_hint = When someone mentions you
settings_notif_quote = Quotes
settings_notif_quote_hint = When someone quotes your post
settings_notif_follow = New followers
settings_notif_follow_hint = When someone follows you
settings_desktop_title = Desktop notifications
settings_desktop_desc = Shown as a system notification only while this tab is in the background.
settings_desktop_state = Permission: { $state }
settings_desktop_allowed = Allowed
settings_desktop_blocked = Blocked
settings_desktop_not_asked = Not asked
settings_desktop_enable = Enable
settings_desktop_blocked_toast = Notifications are blocked for this site in your browser settings.
settings_desktop_unsupported = This browser doesn’t support desktop notifications.
settings_theme_title = Theme
settings_appearance_note = Saved on this device and applied at once.
settings_theme_mode = Theme
settings_theme_system = System
settings_theme_light = Light
settings_theme_dark = Dark
settings_theme_ink = Ink
settings_theme_contrast = Contrast
settings_contrast_normal = Normal
settings_contrast_more = More
settings_reading_title = Text and motion
settings_theme_text_size = Text size
settings_theme_motion = Motion
settings_motion_system = System
settings_motion_reduce = Reduce
settings_preview_title = Preview
settings_preview_text = This is how posts will read with your settings.
settings_about_domain = Domain
settings_about_version = Version
settings_about_contact = Contact
settings_about_terms = Terms of Service
settings_about_privacy = Privacy Policy
settings_about_source = Source code
settings_about_rules = Rules
explore_search_label = Search posts, people and tags
explore_search_placeholder = Search posts, people, #tags
explore_clear = Clear search
explore_tips = Search tips
explore_tips_title = Search operators
explore_op_from = Posts by an account
explore_op_to = Replies or mentions to an account
explore_op_media = Posts with images (or has:poll, has:links)
explore_op_since = Posted on or after a date
explore_op_until = Posted on or before a date
explore_op_phrase = Words in this order
explore_op_exclude = Leave out posts with a word
explore_results = Search results
explore_tab_posts = Posts
explore_tab_people = People
explore_tab_media = Media
explore_people = People
explore_posts = Posts
explore_recent = Recent searches
explore_recent_clear = Clear
explore_trending = Trending
explore_trend_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
explore_trends_empty = No trends yet. Use a #hashtag!
explore_suggestions = Who to follow
explore_suggestions_empty = You follow everyone. Impressive.
explore_no_posts_title = No posts for “{ $q }”
explore_no_posts_body = Try a different word or hashtag.
explore_no_people_title = No people for “{ $q }”
explore_no_people_body = Try a name, a handle, or @handle@instance for someone on another instance.
explore_no_media_title = No images for “{ $q }”
explore_no_media_body = Posts with images that match will show up here.
notif_filter = Notification filter
notif_tab_all = All
notif_tab_mentions = Mentions
notif_empty_all_title = No notifications yet
notif_empty_all_body = Likes, replies, reposts and new followers will show up here.
notif_empty_mentions_title = No mentions yet
notif_empty_mentions_body = Replies, mentions and quotes of your posts show up here.
notif_who_two = { $first } and { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } and { $count } other
       *[other] { $first }, { $second } and { $count } others
    }
notif_verb_like =
    { $count ->
        [one] liked your post
       *[other] liked your post
    }
notif_verb_repost =
    { $count ->
        [one] reposted your post
       *[other] reposted your post
    }
notif_verb_follow =
    { $count ->
        [one] followed you
       *[other] followed you
    }
notif_verb_follow_request =
    { $count ->
        [one] asked to follow you
       *[other] asked to follow you
    }
notif_verb_reply =
    { $count ->
        [one] replied to you
       *[other] replied to you
    }
notif_verb_mention =
    { $count ->
        [one] mentioned you
       *[other] mentioned you
    }
notif_verb_quote =
    { $count ->
        [one] quoted your post
       *[other] quoted your post
    }
notif_verb_poll =
    { $count ->
        [one] ended a poll you voted in
       *[other] ended a poll you voted in
    }
notif_verb_affiliation =
    { $count ->
        [one] invited you to be affiliated with them
       *[other] invited you to be affiliated with them
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Schedule
composer_post_all = Post all
composer_err_type = That file isn’t an image or video tuita can use (PNG, JPEG, GIF, WebP, MP4, WebM or MOV).
composer_err_image_size = Image is too large (max 10 MB).
composer_err_video_size = Videos can be up to { $mb } MB.
composer_err_max_images = You can attach up to 4 images.
composer_err_exclusive = A post can have a video, images or a poll, but only one of them.
composer_extra_placeholder = Add another post
composer_extra_label = Thread post { $n }
composer_extra_remove = Remove thread post { $n }
composer_schedule_chip = Will post { $time }
composer_schedule_clear = Don’t schedule
composer_add_images = Add images
composer_add_video = Add a video
composer_add_poll = Add a poll
composer_add_post = Add another post
composer_scheduled = Scheduled for { $time }
composer_thread_sent = Thread of { $count } posts sent
composer_remove_video = Remove video
composer_remove_image = Remove image
composer_uploading = Uploading
composer_uploading_video = Uploading video… { $percent }%
composer_alt_video = Describe this video
composer_alt_image = Describe this image
composer_alt_label = Description (alt text) of attachment { $index }
composer_emoji = Add emoji
poll_length_hours = { $n } h
poll_length_days = { $n } d
poll_choice = Choice { $n }
poll_choice_optional = Choice { $n } (optional)
poll_remove_choice = Remove choice { $n }
poll_add_choice = Add a choice
poll_length = Poll length
poll_multiple_label = Allow multiple choices
poll_remove = Remove poll
schedule_title = Schedule post
schedule_hour = In an hour
schedule_evening = This evening
schedule_tomorrow = Tomorrow morning
schedule_week = In a week
schedule_window = Pick a time between a minute and 90 days from now.
schedule_custom = Pick a date and time
schedule_set = Set
schedule_clear = Don’t schedule
schedule_view = View scheduled posts
dropzone_drop = Drop to attach
dropzone_reject = Can’t use this file
nav_messages = Messages
nav_lists = Lists
nav_scheduled = Scheduled
notify_message = @{ $handle } sent you a message
shortcut_go_messages = Messages
shortcut_go_lists = Lists
cropper_stage = Crop area. Drag to move the picture. Use the arrow keys to nudge it and plus and minus to zoom.
cropper_hint = Drag to reposition. Pinch, scroll or use the slider to zoom.
cropper_zoom = Zoom
cropper_zoom_in = Zoom in
cropper_zoom_out = Zoom out
cropper_rotate = Rotate
cropper_reset = Reset
cropper_apply = Apply
image_field_avatar = Profile picture
image_field_banner = Banner
image_field_edit_avatar = Edit profile picture
image_field_edit_banner = Edit banner
image_field_change = Change
image_field_remove = Remove
image_field_hint = PNG, JPEG, GIF or WebP up to 10 MB. A GIF becomes a still picture.
image_field_not_image = That file isn’t an image tuita can use (PNG, JPEG, GIF or WebP).
image_field_too_large = Image is too large (max 10 MB).
image_field_uploading = Uploading
image_field_saved = Picture saved.
image_field_removed = Picture removed.
edit_title = Edit post
edit_note = You can edit for 30 minutes after posting. People can see that a post was edited, and its earlier versions.
edit_save = Save
edit_saved = Post updated
edit_edited = Edited
edit_history_title = Edit history
edit_history_current = Current version
edit_history_earlier = Earlier version
edit_history_original = Original
report_post_title = Report post
report_user_title = Report { $handle }
report_intro = What’s wrong? Moderators of this instance will review your report. { $handle } won’t see who reported them.
report_reason = Reason
report_note = Anything else? (optional)
report_send = Send report
report_sent = Thanks. Your report was sent to the moderators.
report_duplicate = You already reported this. The moderators will take a look.
report_spam = Spam
report_spam_body = Unwanted ads, scams, repetitive or misleading content
report_harassment = Harassment
report_harassment_body = Targeting, bullying or threatening someone
report_hate = Hate
report_hate_body = Attacks people for who they are
report_violence = Violence or self-harm
report_violence_body = Threats, glorifying violence, encouraging self-harm
report_misinformation = Misinformation
report_misinformation_body = False claims that could cause harm
report_impersonation = Impersonation
report_impersonation_body = Pretending to be someone else
report_other = Something else
report_other_body = Breaks the rules in another way
post_edit = Edit post
post_edit_history = Edit history
post_report = Report post
post_hide_reply = Hide reply
post_unhide_reply = Unhide reply
post_share_menu = Share…
post_reply_hidden = Hidden reply
post_reply_hidden_toast = Reply hidden
post_reply_unhidden_toast = Reply unhidden
sensitive_label = Sensitive media
sensitive_show = Show
sensitive_hide = Hide sensitive media
media_label =
    { $count ->
        [one] { $count } attachment
       *[other] { $count } attachments
    }
media_video = Video
media_open = Open image { $index } of { $count }
lightbox_title = Media viewer
lightbox_prev = Previous
lightbox_next = Next
poll_vote = Vote
poll_your_vote = Your vote
poll_votes =
    { $count ->
        [one] { $compact } vote
       *[other] { $compact } votes
    }
poll_final = Final results
poll_multiple = multiple choice
poll_left_minutes = { $n } min left
poll_left_hours = { $n } h left
poll_left_days = { $n } d left
pull_refresh = Pull to refresh
messages_title = Messages
messages_new = New message
messages_label = Conversations
messages_empty_title = Start a conversation
messages_empty_body = Messages you send and receive show up here.
messages_you_prefix = You: { $text }
messages_unread = { $count } unread
messages_new_search = Search people
messages_new_results = People
messages_new_none = No one found.
chat_load_older = Load earlier messages
chat_intro = This is the start of your conversation.
chat_label = Messages with { $name }
chat_delete = Delete
chat_seen = Seen
chat_delete_title = Delete this message?
chat_delete_body = It disappears for both of you.
chat_doc_title = Chat with { $name }
chat_error_title = Couldn’t open this conversation
chat_placeholder = Write a message
chat_input_label = Message
chat_send = Send
lists_title = Lists
lists_new = New list
lists_edit = Edit list
lists_label = Your lists
lists_empty_title = No lists yet
lists_empty_body = Group people into lists to read their posts on their own.
lists_members =
    { $count ->
        [one] { $compact } member
       *[other] { $compact } members
    }
lists_private = Private
lists_name = Name
lists_description = Description
lists_private_switch = Private list
lists_save = Save
lists_picker_title = Add { $handle } to lists
lists_picker_none = You have no lists yet.
lists_delete = Delete list
lists_delete_title = Delete this list?
lists_delete_body = “{ $name }” will be removed. The people in it are not affected.
lists_by = List by { $handle }
lists_tab_posts = Posts
lists_tab_members = Members
lists_tabs = List sections
lists_error_title = Couldn’t open this list
lists_posts_label = List posts
lists_posts_empty_title = Nothing here yet
lists_posts_empty_body = Posts from the members of this list show up here.
lists_members_label = List members
lists_members_empty_title = No members
lists_members_empty_body = Add people from their profiles.
lists_remove = Remove
scheduled_title = Scheduled posts
scheduled_label = Scheduled posts
scheduled_status_pending = Scheduled
scheduled_status_published = Posted
scheduled_status_failed = Failed
scheduled_cw = Content warning: { $text }
scheduled_poll = Poll with { $count } choices, runs { $hours } h
scheduled_change_time = Change time
scheduled_post_now = Post now
scheduled_remove = Remove
scheduled_window_error = Pick a time between a minute and 90 days from now.
scheduled_when = Post at
scheduled_zone = Time zone: { $zone }
scheduled_save = Save
scheduled_posted = Posted.
scheduled_cancel_title = Cancel this scheduled post?
scheduled_cancel_body = It will not be posted.
scheduled_cancel_confirm = Cancel post
scheduled_empty_title = Nothing scheduled
scheduled_empty_body = Posts you schedule wait here until their time.
profile_message = Message
profile_add_to_lists = Add to lists
media_alt = ALT
profile_report = Report { $handle }
chart_show_table = Show { $title } as a table
chart_col_day = Day
chart_col_hour = Hour (UTC)
chart_empty = { $title }: nothing to show yet.
chart_bar_summary = { $title }: { $total } in total, most on { $label } ({ $peak }).
chart_spark_summary = { $title }: { $total } in total, { $last } on { $label }.
chart_heat_summary = { $title }: { $total } in total, busiest on { $label } at { $hour } ({ $peak }).
chart_heat_less = Less
chart_heat_more = More
announce_region = Announcements
announce_dismiss = Dismiss announcement
announce_more = Read more
announce_tone_info = Notice
announce_tone_success = Good news
announce_tone_warning = Warning
admin_stat_people = People
admin_stat_people_sub = { $count } active this week
admin_stat_posts_today = Posts, 24 h
admin_stat_posts_total = { $count } in total
admin_stat_reports = Open reports
admin_stat_reports_open = Needs review
admin_stat_reports_clear = All clear
admin_stat_signups_today = Sign-ups today
admin_stat_signups_sub = { $count } in 14 days
admin_stat_suspended = Suspended
admin_stat_suspended_sub = accounts
admin_chart_posts = Posts per day
admin_chart_signups = Sign-ups per day
admin_chart_reports = Reports per day
admin_chart_heat = Activity by weekday and hour
nav_admin = Admin
admin_title = Admin
admin_sections = Admin sections
admin_tab_overview = Overview
admin_tab_reports = Reports
admin_tab_users = People
admin_tab_instance = Instance
admin_tab_federation = Federation
admin_tab_announcements = Announcements
admin_tab_invites = Invites
admin_tab_filter = Content filter
admin_tab_audit = Audit log
admin_denied_title = Staff only
admin_denied_body = This area is for the admins and moderators of this instance.
admin_denied_home = Go home
admin_unknown = unknown
admin_tag_active = Active
admin_tag_suspended = Suspended
admin_tag_deleted = Deleted
admin_tag_org = Organization
admin_audit_system = system
admin_audit_empty = Nothing logged yet
admin_report_status = Report status
admin_report_open = Open
admin_report_resolved = Resolved
admin_reports_empty_open = No open reports
admin_reports_empty_open_body = When people report posts or accounts, they show up here.
admin_reports_empty_resolved = Nothing resolved yet
admin_report_by = reported by { $handle }
admin_report_same_target = { $count } open reports
admin_report_media =
    { $count ->
        [one] { $count } attachment
       *[other] { $count } attachments
    }
admin_report_poll = Has a poll
admin_report_dismiss = Dismiss
admin_report_delete_post = Delete post
admin_report_suspend = Suspend account
admin_report_delete_suspend = Delete and suspend
admin_report_confirm_title = Take action?
admin_report_confirm_body = { $action }. This is recorded in the audit log.
admin_report_confirm = Confirm
admin_report_resolution = { $action } by { $handle } · { $time }
admin_res_dismiss = Dismissed
admin_res_delete_post = Post deleted
admin_res_suspend_user = Account suspended
admin_res_delete_and_suspend = Post deleted and account suspended
admin_users_search = Search people
admin_users_search_placeholder = Name or handle
admin_users_show = Show
admin_users_empty = No people found
admin_filter_local = Local
admin_filter_remote = Remote
admin_filter_staff = Staff
admin_filter_verified = Verified
admin_filter_suspended = Suspended
admin_filter_all = Everyone
admin_user_details = Details for { $handle }
admin_user_menu = Manage { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
admin_user_followers =
    { $count ->
        [one] { $compact } follower
       *[other] { $compact } followers
    }
admin_user_joined = joined { $time }
admin_user_active = active { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } open report
       *[other] { $compact } open reports
    }
admin_act_verify = Verify
admin_act_unverify = Remove verification
admin_act_org = Make organization
admin_act_unorg = Remove organization status
admin_act_role_user = Remove staff role
admin_act_role_moderator = Make moderator
admin_act_role_admin = Make admin
admin_act_suspend = Suspend
admin_act_unsuspend = Unsuspend
admin_act_purge = Delete all posts
admin_act_rename = Change handle
admin_act_reset_name = Reset display name
admin_act_clear_bio = Clear bio
admin_act_delete = Delete account
admin_done_verified = { $handle } is verified
admin_done_unverified = Verification removed
admin_done_org = { $handle } is now an organization
admin_done_unorg = Organization status removed
admin_done_role = Role updated
admin_done_suspended = { $handle } suspended
admin_done_unsuspended = { $handle } unsuspended
admin_done_purged =
    { $count ->
        [one] { $count } post deleted
       *[other] { $count } posts deleted
    }
admin_done_name_reset = Display name reset
admin_done_bio_cleared = Bio cleared
admin_done_deleted = { $handle } deleted
admin_done_renamed = Now { $handle }
admin_role_phrase_user = a regular user
admin_role_phrase_moderator = a moderator
admin_role_phrase_admin = an admin
admin_role_title = Change the role of { $handle }?
admin_role_body = They will become { $role }.
admin_role_confirm = Change role
admin_unorg_title = Remove organization status from { $handle }?
admin_unorg_body = All of its affiliations end.
admin_unorg_confirm = Remove
admin_suspend_title = Suspend { $handle }?
admin_suspend_body = They are signed out and their content stays hidden until you unsuspend them.
admin_suspend_confirm = Suspend
admin_purge_title = Delete every post by { $handle }?
admin_purge_body = This can’t be undone.
admin_purge_confirm = Delete all
admin_reset_name_title = Reset the display name of { $handle }?
admin_reset_name_body = It will be replaced with their handle.
admin_reset_name_confirm = Reset
admin_delete_title = Delete { $handle }?
admin_delete_body = Their posts, follows and messages are removed and the account is anonymised. This can’t be undone.
admin_delete_confirm = Delete account
admin_rename_title = Change the handle of { $handle }
admin_rename_note = Links to the old handle stop working.
admin_rename_handle = New handle
admin_rename_reset_name = Also reset the display name
admin_rename_save = Change handle
admin_detail_title = Account details
admin_detail_invite = Invite { $code }
admin_detail_posts = Posts
admin_detail_followers = Followers
admin_detail_following = Following
admin_detail_open_reports = Open reports
admin_detail_sessions = Sessions
admin_detail_passkeys = Passkeys
admin_detail_tokens = API tokens
admin_detail_joined = Joined { $time }
admin_detail_recent_posts = Recent posts
admin_detail_no_posts = No posts.
admin_detail_reports = Reports about this account
admin_detail_no_reports = No reports.
admin_detail_history = Moderation history
admin_detail_nothing = Nothing yet.
admin_instance_title = Instance
admin_instance_name = Instance name
admin_instance_description = Description
admin_instance_contact = Contact email
admin_instance_contact_hint = Shown in the Terms and Privacy pages
admin_instance_min_age = Minimum age
admin_instance_rules = Rules
admin_instance_rules_hint = One rule per line
admin_instance_preview = Preview the terms
admin_instance_save = Save
admin_instance_saved = Instance settings saved
admin_instance_template_note = The Terms of Service and Privacy Policy are templates that use these values. Have them reviewed for your jurisdiction before you open the instance to the public.
admin_signups_title = Sign-ups
admin_signups_desc = Who may create an account on this instance
admin_signups_open = Open
admin_signups_open_hint = Anyone can create an account.
admin_signups_invite = Invite only
admin_signups_invite_hint = People need an invite code from the Invites tab.
admin_signups_closed = Closed
admin_signups_closed_hint = Nobody can sign up.
admin_filter_title = Content filter
admin_filter_desc = Nothing is blocked until you add words below. Blocked words apply to handles, display names, locations, bios, posts and list names, including disguised spellings. Reserved handles like “admin” can’t be registered.
admin_filter_blocked = Blocked words
admin_filter_blocked_hint = One per line, matched inside words
admin_filter_allowed = Allowed words
admin_filter_allowed_hint = Exceptions to the blocked words, one per line
admin_filter_reserved = Extra reserved handles
admin_filter_reserved_hint = One per line
admin_filter_save = Save filter
admin_filter_saved = Filter saved
admin_filter_field_handle = handle
admin_filter_field_name = name
admin_filter_field_location = location
admin_filter_field_bio = bio
admin_scan_title = Scan accounts
admin_scan_desc = Find accounts whose handle, name or bio break the filter.
admin_scan_run = Scan now
admin_scan_clean = No accounts break the filter.
admin_scan_found =
    { $count ->
        [one] { $count } account needs attention
       *[other] { $count } accounts need attention
    }
admin_scan_reason = The { $field } contains { $reason }
admin_ann_title = New announcement
admin_ann_desc = A banner at the top of everyone’s screen until it expires or you end it. People can dismiss it.
admin_ann_text = Message
admin_ann_link = Link
admin_ann_link_hint = Optional: https://… or /path
admin_ann_tone = Style
admin_ann_tone_info = Info
admin_ann_tone_success = Good news
admin_ann_tone_warning = Warning
admin_ann_duration = Duration
admin_ann_days =
    { $count ->
        [one] { $count } day
       *[other] { $count } days
    }
admin_ann_week = 1 week
admin_ann_until_ended = Until ended
admin_ann_publish = Publish
admin_ann_published = Announcement published
admin_ann_list = Announcements so far
admin_ann_none = No announcements yet.
admin_ann_end = End
admin_ann_ended = ended
admin_ann_expired = expired
admin_ann_inactive = Not showing
admin_ann_until = until { $date }
admin_inv_title = Invite codes
admin_inv_desc = Let specific people join while sign-ups are invite only.
admin_inv_note = Note (who it’s for)
admin_inv_uses = Uses
admin_inv_use_count =
    { $count ->
        [one] { $count } use
       *[other] { $count } uses
    }
admin_inv_expiry = Expires
admin_inv_week = In 1 week
admin_inv_days =
    { $count ->
        [one] In { $count } day
       *[other] In { $count } days
    }
admin_inv_never = Never
admin_inv_create = Create code
admin_inv_created = Code { $code } created
admin_inv_created_copied = Code { $code } created and copied
admin_inv_list = Codes
admin_inv_none = No invite codes yet.
admin_inv_state_active = Active
admin_inv_state_revoked = Revoked
admin_inv_state_used = Used up
admin_inv_state_expired = Expired
admin_inv_used = { $uses } of { $max } used
admin_inv_expires = expires { $date }
admin_inv_copy = Copy code { $code }
admin_inv_copied = Code copied
admin_inv_copy_failed = Couldn’t copy. The code is { $code }.
admin_inv_revoke = Revoke
admin_inv_revoke_title = Revoke this invite?
admin_inv_revoke_body = People who already joined with it keep their accounts.
settings_devices = Devices & sessions
settings_devices_desc = See where you’re signed in and sign out remotely
settings_passkeys = Passkeys
settings_passkeys_desc = Sign in with your fingerprint, face, screen lock or a security key
settings_affiliation = Affiliation
settings_affiliation_desc = Organizations you’re part of, or your organization’s accounts
settings_bots = Bots & API
settings_bots_desc = API tokens and the “Automated” label
settings_saved = Saved
passkey_title = Your passkeys
passkey_desc = Sign in with your fingerprint, face, screen lock or security key instead of a password. Passkeys can’t be phished, and nothing secret is stored on our server.
passkey_none = No passkeys yet.
passkey_add = Add a passkey
passkey_added = Passkey “{ $name }” added
passkey_default_name = Passkey
passkey_added_ago = added { $time }
passkey_used_ago = last used { $time }
passkey_never_used = never used
passkey_rename = Rename
passkey_rename_label = Rename passkey { $name }
passkey_rename_title = Rename passkey
passkey_name = Name
passkey_rename_save = Save
passkey_remove = Remove
passkey_remove_label = Remove passkey { $name }
passkey_remove_title = Remove “{ $name }”?
passkey_remove_body = You won’t be able to sign in with this passkey any more. You may also want to delete it from your device or password manager.
passkey_removed = Passkey removed
passkey_unsupported = This browser can’t use passkeys on this address.
passkey_needs_domain = Passkeys need this site to have a domain name, not an IP address.
passkey_already_here = This device already has a passkey for this account.
token_scope_read = Read
token_scope_write = Post and interact
token_scope_messages = Direct messages
token_scope_read_hint = Timelines, notifications, profiles, search and the realtime stream
token_scope_write_hint = Posting, deleting, likes, reposts, bookmarks, follows and uploads
token_scope_messages_hint = Reading and sending direct messages
token_title = API tokens
token_desc = Tokens let scripts and bots use your account through the API. Treat them like passwords: anyone with a token can act as you within its scopes.
token_none = No tokens yet.
token_name = Token name
token_name_hint = For example: weather bot
token_scopes = Scopes
token_create = Create token
token_used_ago = last used { $time }
token_never_used = never used
token_revoke = Revoke
token_revoke_label = Revoke token { $name }
token_revoke_title = Revoke “{ $name }”?
token_revoke_body = Anything using it stops working.
token_revoked = Token revoked
token_secret_title = Your new token
token_secret_warning = Copy it now. For your security it won’t be shown again.
token_secret_label = New API token
token_copy = Copy
token_copied = Token copied
token_copy_failed = Couldn’t copy. Select the token and copy it by hand.
token_done = Done
bot_title = Automated account
bot_desc = Shows an “Automated” label next to the name, so people know a bot is posting.
bot_switch = This is an automated account
bot_owner = Run by
bot_owner_hint = The handle of the person or organization behind it. Optional.
bot_owner_save = Save
bot_docs = Developer docs
bot_docs_desc = Endpoints, realtime events and example bots
aff_yours_title = Your affiliation
aff_yours_desc = An organization can invite you to show its badge next to your name. Nothing changes until you accept, and you can leave at any time.
aff_none = You’re not affiliated with an organization.
aff_invites_title = Invitations
aff_accept = Accept
aff_decline = Decline
aff_leave = Leave
aff_leave_title = Leave this organization?
aff_leave_body = Its badge leaves your profile.
aff_remove_title = Remove { $handle }?
aff_remove_body = Your badge leaves their profile.
aff_remove = Remove
aff_remove_label = Remove { $handle }
aff_ended = Affiliation ended
aff_declined = Invitation declined
aff_accepted = Affiliation added
aff_invited = Invitation sent
aff_org_title = Your affiliates
aff_org_desc = Invite people to show your badge next to their name. They have to accept.
aff_handle = Handle
aff_label = Label
aff_label_hint = Optional, like “Engineer”
aff_invite = Send invitation
aff_members_none = No affiliates yet.
aff_pending = Invited
aff_not_org = Are you an organization? Ask an admin of this instance to mark your account as one.
auth_passkey = Sign in with a passkey
auth_or = or
auth_invite = Invite code
auth_invite_hint = This instance takes new people by invitation only.
auth_invite_required = Enter the invite code you were given.
landing_developers = Developers
landing_invite = Sign-ups are by invitation only. Have a code? Create your account.
dev_title = Developers
dev_hero = Bots & API
dev_hero_sub = Build bots, scripts and integrations for { $name }
dev_intro_h = Build for { $name }
dev_intro_p = Everything the tuita apps do goes through a small JSON API, and bots and scripts can use it too. You need an account on { $name } and a personal API token. There’s no app registration and no OAuth dance.
dev_intro_1 = Create an account for your bot, or use your own.
dev_intro_2 = In Settings, Bots & API, turn on “This is an automated account” so people see an “Automated” label, and say who runs it.
dev_intro_3 = Create a token with the scopes you need and keep it secret.
dev_auth_h = Authentication
dev_auth_p = Send the token in the Authorization header. Every request and response is JSON (UTF-8), except uploads, which are multipart.
dev_auth_prefix = A token is tuita_ followed by 43 letters, digits, dashes and underscores. Only a hash is stored, so a lost token can’t be shown again: make a new one. A token has one or more of these scopes:
dev_auth_refused = A token can’t change passwords, sessions, tokens or passkeys, can’t export or delete the account, and can’t use the admin API. Those need a real sign-in on the website and answer 403.
dev_auth_errors = Errors come back as JSON with an error message and a 4xx status. 401 means the token is wrong or revoked, 403 a missing scope or a refused route, and 429 comes with a Retry-After header.
dev_post_h = Your first post
dev_post_p = Upload images and video first with POST /api/v1/media (multipart), then attach them by id in mediaIds. A reply sets replyTo, a quote sets quoteOf.
dev_endpoints_h = Endpoints
dev_endpoints_intro = Paths in curly braces are placeholders. Lists page with ?before= set to the next value of the previous page; limit is 1 to 40. The scope column is the scope a token needs.
dev_col_method = Method
dev_col_path = Path
dev_col_scope = Scope
dev_col_what = What it does
dev_ep_me = The account the token belongs to
dev_ep_patch_me = Change the profile and settings of that account
dev_ep_bot = Turn the “Automated” label on or off and name the person who runs the bot
dev_ep_following = The home timeline of the people you follow
dev_ep_foryou = The For you timeline; page with ?cursor=
dev_ep_notifications = Notifications (?filter=mentions for replies, mentions and quotes)
dev_ep_post = A post with its thread and replies
dev_ep_user = A profile (handle@domain for accounts on other servers)
dev_ep_user_posts = Someone’s posts (?tab=replies, media or likes; with media, ?kind=photo or video)
dev_ep_search = Search posts, people or media, with operators like from: and has:media
dev_ep_trends = The tags used most in the last week
dev_ep_stream = Realtime events (Server-Sent Events)
dev_ep_create = Write a post
dev_ep_thread = Write 2 to 25 posts as a thread
dev_ep_delete = Delete one of your posts
dev_ep_like = Like a post (DELETE to undo). Also /repost and /bookmark
dev_ep_follow = Follow someone (DELETE to unfollow)
dev_ep_media = Upload an image or video as multipart; the answer holds the id to attach
dev_ep_schedule = Schedule a post for later
dev_ep_report = Report a post or an account to the moderators
dev_ep_conversations = Your direct message conversations
dev_ep_message = Send a direct message; the body is JSON with a text field
dev_realtime_h = Realtime events
dev_realtime_p = GET /api/v1/stream is a Server-Sent Events stream. It sends a notification event for every like, repost, follow, reply, mention or quote, a message event for direct messages and badges with unread counts. It sends a comment line every 25 seconds and asks clients to retry after 5 seconds; reconnect if the connection drops.
dev_example_h = Example: a bot that answers mentions
dev_example_node = Node 22 or newer, no packages:
dev_example_python = The same in Python, using only requests:
dev_limits_h = Limits
dev_limit_rate = Each token may make 600 requests per 10 minutes. Over the limit the server answers 429 with a Retry-After header.
dev_limit_tokens = An account can have up to 20 tokens and create up to 20 tokens per hour.
dev_limit_posting = Posting, following, reporting and the other actions keep their own limits per account.
dev_rules_h = Rules for bots
dev_rule_1 = Turn on the Automated label, and say who runs the bot.
dev_rule_2 = Only reply or message people who asked for it: they mentioned the bot, followed it or messaged it first. Don’t send unsolicited mentions or DMs.
dev_rule_3 = Keep it calm: stay well under the limits above.
dev_rule_4 = Follow the instance’s Terms and rules. Moderators can suspend bots like any account.
admin_act_log_admin_bootstrap = First admin set up
admin_act_log_announcement_create = Announcement published
admin_act_log_announcement_end = Announcement ended
admin_act_log_instance_settings = Instance settings changed
admin_act_log_invite_create = Invite created
admin_act_log_invite_revoke = Invite revoked
admin_act_log_post_delete = Post deleted
admin_act_log_report_dismiss = Report dismissed
admin_act_log_user_delete = Account deleted
admin_act_log_user_org = Marked as organization
admin_act_log_user_unorg = Organization status removed
admin_act_log_user_purge_posts = All posts deleted
admin_act_log_user_rename = Account renamed
admin_act_log_user_role = Role changed
admin_act_log_user_suspend = Account suspended
admin_act_log_user_unsuspend = Account unsuspended
admin_act_log_user_unverify = Verification removed
admin_act_log_user_verify = Account verified
admin_act_log_domain_block = Instance blocked
admin_act_log_domain_unblock = Instance unblocked
admin_act_log_federation_retry = Delivery retry requested
admin_fed_stat_domains = Known instances
admin_fed_stat_domains_sub = seen by this instance
admin_fed_stat_accounts = Remote accounts
admin_fed_stat_accounts_sub = known from elsewhere
admin_fed_stat_posts = Remote posts
admin_fed_stat_posts_sub = stored locally
admin_fed_stat_queue = Queued
admin_fed_stat_queue_sub = deliveries waiting
admin_fed_queue_title = Delivery queue
admin_fed_queue_desc = Signed activities waiting to reach other instances. Failed ones retry with growing delays and are dropped after 12 attempts.
admin_fed_queue_empty = Nothing is waiting. Every delivery has been sent.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } delivery waiting
       *[other] { $compact } deliveries waiting
    }
admin_fed_queue_due = { $count } due now
admin_fed_queue_oldest = Oldest queued { $date }
admin_fed_retry = Retry now
admin_fed_retry_done =
    { $count ->
        [one] { $compact } delivery will be retried now.
       *[other] { $compact } deliveries will be retried now.
    }
admin_fed_failing = Failing instances
admin_fed_failing_none = No instance is failing.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } delivery
       *[other] { $compact } deliveries
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } attempt
       *[other] { $compact } attempts
    }
admin_fed_failing_next = next try { $date }
admin_fed_failing_error = Last error
admin_fed_failing_no_error = No error recorded
admin_fed_actor_title = Instance actor
admin_fed_actor_desc = Other instances check the requests this instance signs against this key.
admin_fed_actor_id = Actor
admin_fed_actor_key = Key id
admin_fed_actor_fingerprint = SHA-256 fingerprint
admin_fed_actor_copy = Copy fingerprint
admin_fed_actor_copied = Fingerprint copied
admin_fed_actor_copy_failed = Couldn’t copy. The fingerprint is { $fingerprint }
admin_fed_blocks_title = Blocked instances
admin_fed_blocks_desc = Silence hides an instance’s accounts from public timelines and search. Suspend hides them everywhere, drops what they send and stops deliveries to them.
admin_fed_domain = Domain
admin_fed_domain_hint = example.social, or a link to any page on it
admin_fed_severity = Severity
admin_fed_silence = Silence
admin_fed_suspend = Suspend
admin_fed_reason = Reason
admin_fed_reason_hint = Optional, shown to staff only
admin_fed_block = Block instance
admin_fed_blocked = { $domain } is blocked
admin_fed_none = No instances are blocked.
admin_fed_list = Blocks
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } known account
       *[other] { $compact } known accounts
    }
admin_fed_row_no_reason = No reason given
admin_fed_row_changed = { $domain } is now set to { $severity }
admin_fed_remove = Remove
admin_fed_remove_label = Remove the block on { $domain }
admin_fed_remove_title = Remove the block on { $domain }?
admin_fed_remove_body = Its accounts and posts come back and deliveries to it start again.
admin_fed_removed = { $domain } is no longer blocked
remote_domain_title = Account on { $domain }
remote_view_original = View on { $domain }
visibility_unlisted = Unlisted
visibility_unlisted_hint = Not shown on public timelines
visibility_followers = Followers only
visibility_followers_hint = Only followers can see this
visibility_direct = Direct
visibility_direct_hint = Only the people mentioned can see this
follow_requested = Requested
follow_cancel_request = Cancel follow request
follow_requested_toast = Follow request sent to { $handle }
follow_cancelled_toast = Follow request cancelled
profile_locked = Approves followers
profile_locked_hint = Follow requests need this account’s approval.
profile_original = View profile on { $domain }
profile_remote_note = A copy from { $domain }. Counts and posts are what this instance has seen.
remote_lookup_loading = Looking up { $handle } on { $domain }…
remote_lookup_notfound_title = No such account
remote_lookup_notfound_body = { $handle } doesn’t exist, or { $domain } doesn’t share it.
remote_lookup_failed_title = Couldn’t load { $handle }
explore_lookup = Looking up { $target }…
explore_lookup_empty_body = That account or post wasn’t found. The other instance may be offline, or it may not share it.
language_label = Language
oauth_title = Authorize an app
oauth_loading = Checking the request…
oauth_wants = This app wants to use your account { $handle }.
oauth_name_warning = Anyone can pick any app name. Approve only apps you trust.
oauth_website = Website
oauth_allow = It will be able to
oauth_scope_read = Read your profile, posts, follows, lists, timelines and notifications, and direct messages that are not end-to-end encrypted.
oauth_scope_write = Post, edit and delete for you, like, follow, block, mute and change your profile and settings.
oauth_scope_messages = Read and send your direct messages.
oauth_push_note = Push notifications aren’t supported. An app that asks for them does not get them.
oauth_redirect = After you answer, you’re sent to
oauth_redirect_oob = The app gets no redirect. A code is shown here for you to paste into it.
oauth_approve = Authorize
oauth_deny = Deny
oauth_switch = Use another account
oauth_sent_approved = Authorized. Sending you back to { $host }. If nothing happens, close this tab.
oauth_sent_denied = Denied. Sending you back to { $host }. If nothing happens, close this tab.
oauth_denied = Denied. Nothing was shared with the app.
oauth_code_title = Authorization code
oauth_code_body = Paste this code into the app. It works once and expires in 10 minutes.
oauth_code_copy = Copy code
oauth_code_copied = Code copied
oauth_code_copy_failed = Couldn’t copy. Select the code and copy it by hand.
oauth_error_title = This request can’t be authorized
oauth_error_note = Nothing was sent to the app.
oauth_error_invalid_request = The request is malformed, or it asks to return to an address the app didn’t register. Go back to the app and try again.
oauth_error_invalid_client = This app isn’t registered here.
oauth_error_invalid_scope = The app asks for access that tuita doesn’t offer, or more than it registered for.
oauth_error_unsupported_response_type = The app asks for a kind of sign-in that tuita doesn’t support.
oauth_error_rate = You’ve answered too many requests. Wait a few minutes and try again.
oauth_error_unreachable = Couldn’t reach tuita. Check your connection and try again.
oauth_home = Go to tuita
share_send_chat = Send via chat
share_download = Download media
share_download_n = Download media { $number }
share_chat_title = Send post via chat
share_chat_sent = Sent to { $handle }
hover_card_label = Profile card for { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Media type
profile_media_all = All media
profile_media_photos = Photos
profile_media_videos = Videos
profile_empty_videos_title = No videos yet
profile_empty_videos_body = Posts with videos will show up here.
account_add_existing = Add an existing account
rail_label = Search and suggestions
rail_search_submit = Search
rail_show_more = Show more
rail_footer = Links
chat_new = New chat
chat_search = Search conversations
chat_search_none = No conversations match your search.
chat_start_title = Start conversation
chat_start_body = Pick a conversation from the list, or start a new one.
explore_sections = Explore sections
explore_see_more = Show more
history_title = History
history_tabs = History sections
history_likes_label = Liked posts
video_play = Play
video_pause = Pause
video_mute = Mute
video_unmute = Unmute
video_volume = Volume
video_seek = Seek
video_position = { $current } of { $total }
video_speed = Playback speed
video_fullscreen = Full screen
video_exit_fullscreen = Exit full screen
video_pip = Picture in picture
video_exit_pip = Exit picture in picture
video_error = This video can’t be played.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Hide panel
lightbox_show_panel = Show panel
lightbox_view_post = View post
lightbox_panel = Post and replies
emotepick_tabs = Emoji sets
emotepick_tab_emoji = Emoji
emotepick_tab_custom = Custom
admin_emote_tab = Emotes
admin_emote_title = Custom emotes
admin_emote_desc = Emotes belong to the whole instance. Anyone can use them as :shortcode: in posts, names and messages. A shortcode can’t be renamed; delete the emote and upload it again.
admin_emote_shortcode = Shortcode
admin_emote_shortcode_hint = 2 to 32 letters, numbers or underscores. Shown as :shortcode:.
admin_emote_invalid = Use only letters, numbers and underscores, 2 to 32 characters.
admin_emote_category = Category
admin_emote_category_hint = Optional. Groups emotes in the picker.
admin_emote_file = Emote image
admin_emote_file_hint = PNG, JPEG, WebP or GIF, up to 256 KB. Drop a file here or choose one.
admin_emote_file_type = That file isn’t a PNG, JPEG, WebP or GIF image.
admin_emote_file_size = Image is too large (max 256 KB).
admin_emote_choose_file = Choose image
admin_emote_change_file = Change image
admin_emote_upload = Upload emote
admin_emote_upload_failed = Couldn’t upload the emote. Try again.
admin_emote_uploaded = Added :{ $shortcode }:.
admin_emote_list = Emotes on this instance
admin_emote_none = No custom emotes yet.
admin_emote_no_category = No category
admin_emote_category_for = Category of :{ $shortcode }:
admin_emote_saved = Category saved.
admin_emote_delete = Delete :{ $shortcode }:
admin_emote_delete_title = Delete this emote?
admin_emote_delete_body = :{ $shortcode }: stops rendering as an image everywhere and shows as plain text. Posts that used it keep the text.
admin_emote_delete_confirm = Delete emote
admin_emote_deleted = Emote deleted.
admin_act_log_emoji_create = Emote added
admin_act_log_emoji_update = Emote changed
admin_act_log_emoji_delete = Emote deleted
e2ee_settings = Encrypted messages
e2ee_settings_desc = Passphrase, this device and reset
e2ee_status_label = Status
e2ee_status_none = Off. Turn it on to send and read end-to-end encrypted messages.
e2ee_status_locked = Locked on this device. Enter your passphrase to read your messages here.
e2ee_status_ready = On and unlocked on this device.
e2ee_keys_error = Couldn’t check your encryption keys.
e2ee_setup_title = Turn on encrypted messages
e2ee_setup_body = Messages to people on this server are sealed in your browser. The server stores only text it can’t read. A passphrase lets you unlock your messages on another device.
e2ee_setup_warning = Your passphrase can’t be recovered. If you forget it you can reset encryption, but your earlier messages are lost.
e2ee_setup_submit = Turn on
e2ee_setup_done = Encrypted messages are on
e2ee_unlock_title = Unlock encrypted messages
e2ee_unlock_body = Enter your passphrase to read and send encrypted messages on this device.
e2ee_unlock_submit = Unlock
e2ee_unlock_wrong = That passphrase is wrong.
e2ee_unlock_done = Unlocked
e2ee_passphrase = Passphrase
e2ee_passphrase_confirm = Confirm passphrase
e2ee_passphrase_hint = At least 10 characters.
e2ee_passphrase_short = Use at least 10 characters.
e2ee_passphrase_mismatch = The passphrases don’t match.
e2ee_failed = Something went wrong. Try again.
e2ee_unsupported = This browser can’t do encrypted messages.
e2ee_forget = Forget on this device
e2ee_forget_hint = Removes your key and the contacts you trust from this browser. Your passphrase unlocks them again.
e2ee_forget_done = Forgotten on this device
e2ee_reset = Reset encryption
e2ee_reset_hint = Makes a new key. Messages you sent and received before become unreadable.
e2ee_reset_title = Reset encryption?
e2ee_reset_body = Your earlier messages become unreadable, for you and for the people you wrote to, and can’t be brought back. Your contacts will be asked to trust your new key. Choose a new passphrase.
e2ee_reset_submit = Reset
e2ee_reset_done = Encryption was reset
e2ee_note_sealed = End-to-end encrypted
e2ee_note_remote = Not end-to-end encrypted: this person is on another server.
e2ee_note_plain = Not end-to-end encrypted.
e2ee_plain_mark = Not encrypted
e2ee_preview = Encrypted message
e2ee_line_sealed = Encrypted message
e2ee_line_own_key = Sent to a key you no longer have
e2ee_line_peer_key = Sent with a key you haven’t trusted
e2ee_line_unreadable = Couldn’t be opened
e2ee_line_unsupported = This browser can’t open encrypted messages
e2ee_block_peer_off = { $handle } hasn’t turned on encrypted messages yet.
e2ee_block_self_off = Turn on encrypted messages in Settings first.
e2ee_block_unlock = Unlock encrypted messages in Settings first.
e2ee_block_downgrade = { $handle } used encrypted messages before. This conversation won’t fall back to unencrypted messages.
e2ee_block_changed = Trust { $handle }’s new key to write to them.
e2ee_block_error = Couldn’t check your encryption keys.
e2ee_changed_title = { $name }’s encryption key changed
e2ee_changed_body = They may have reset encryption or moved to a new device, or someone may be in the way. Compare the new safety number with them before you trust it. You can’t send until you do.
e2ee_changed_old = Safety number you trusted
e2ee_changed_new = New safety number
e2ee_trust = Trust new key
e2ee_safety_open = Safety number
e2ee_safety_title = Safety number
e2ee_safety_body = Compare this number with { $name } in person or on a call, not in this chat. If it matches on both screens, nobody is between you.

## 0031
composer_upload_failed = Upload failed. Try again.
composer_send_failed = Couldn’t reach tuita. Check your connection and try again.
composer_open_failed = Couldn’t open the composer. Try again.
