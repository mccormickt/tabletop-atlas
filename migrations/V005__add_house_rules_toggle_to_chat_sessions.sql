-- Add include_house_rules column to chat_sessions
-- This allows users to toggle whether house rules are included in chat context

ALTER TABLE chat_sessions ADD COLUMN include_house_rules BOOLEAN DEFAULT 1;
