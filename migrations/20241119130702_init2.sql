
CREATE TABLE IF NOT EXISTS covers (
    number integer NOT NULL PRIMARY KEY,
    image bytea,
    size integer
);

CREATE TABLE IF NOT EXISTS cycles (
    number integer DEFAULT 0 NOT NULL PRIMARY KEY,
    german_title character varying(80) DEFAULT NULL::character varying,
    english_title character varying(80) DEFAULT NULL::character varying,
    short_title character varying(40) DEFAULT NULL::character varying,
    start integer,
    "end" integer
);

CREATE TABLE IF NOT EXISTS hefte (
    number integer DEFAULT 0 NOT NULL PRIMARY KEY,
    title character varying(80) DEFAULT NULL::character varying,
    author character varying(60) DEFAULT NULL::character varying,
    published date,
    german_file character varying(100) DEFAULT NULL::character varying
);

CREATE TABLE IF NOT EXISTS pending (
    id integer NOT NULL PRIMARY KEY,
    number integer,
    german_title character varying(80) DEFAULT NULL::character varying,
    author character varying(60) DEFAULT NULL::character varying,
    published character varying(60) DEFAULT NULL::character varying,
    english_title character varying(80) DEFAULT NULL::character varying,
    author_name character varying(60) DEFAULT NULL::character varying,
    author_email character varying(60) DEFAULT NULL::character varying,
    date_summary character varying(40) DEFAULT NULL::character varying,
    summary text
);

CREATE TABLE IF NOT EXISTS summaries (
    number integer DEFAULT 0 NOT NULL PRIMARY KEY,
    english_title character varying(80) DEFAULT NULL::character varying,
    author_name character varying(60) DEFAULT NULL::character varying,
    author_email character varying(60) DEFAULT NULL::character varying,
    date character varying(40) DEFAULT NULL::character varying,
    summary text,
    "time" character varying(20) DEFAULT NULL::character varying
);

CREATE TABLE IF NOT EXISTS summaries_fr (
    number integer DEFAULT 0 NOT NULL,
    english_title character varying(80) DEFAULT NULL::character varying,
    author_name character varying(60) DEFAULT NULL::character varying,
    author_email character varying(60) DEFAULT NULL::character varying,
    date character varying(40) DEFAULT NULL::character varying,
    summary text,
    "time" character varying(20) DEFAULT NULL::character varying
);

CREATE TABLE IF NOT EXISTS users (
    login character varying(40) DEFAULT ''::character varying NOT NULL PRIMARY KEY,
    name character varying(80) DEFAULT NULL::character varying,
    level integer DEFAULT 5,
    email character varying(60) DEFAULT NULL::character varying,
    auth_token text,
    salt bytea,
    password bytea,
    temp_link character varying(60),
    last_login character varying(50)
);
