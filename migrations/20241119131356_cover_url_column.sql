-- Add migration script here

ALTER TABLE public.covers ADD COLUMN url VARCHAR(200);

