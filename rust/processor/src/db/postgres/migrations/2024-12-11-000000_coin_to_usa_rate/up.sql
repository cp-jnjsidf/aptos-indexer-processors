-- Table: public.coin_to_usd_rates

-- DROP TABLE IF EXISTS public.coin_to_usd_rates;

CREATE TABLE IF NOT EXISTS public.coin_to_usd_rates
(
    coin_id character varying(255) COLLATE pg_catalog."default" NOT NULL,
    name character varying(50) COLLATE pg_catalog."default",
    source character varying(50) COLLATE pg_catalog."default" NOT NULL,
    price numeric(20,10) NOT NULL,
    readable_timestamp timestamp without time zone,
    "timestamp" timestamp without time zone NOT NULL,
    CONSTRAINT coin_to_usd_rates_pkey PRIMARY KEY (coin_id, source, "timestamp")
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.coin_to_usd_rates
    OWNER to postgres;
-- Index: idx_coin_id_source_timestamp

-- DROP INDEX IF EXISTS public.idx_coin_id_source_timestamp;

CREATE INDEX IF NOT EXISTS idx_coin_id_source_timestamp
    ON public.coin_to_usd_rates USING btree
    (coin_id COLLATE pg_catalog."default" ASC NULLS LAST, source COLLATE pg_catalog."default" ASC NULLS LAST, "timestamp" ASC NULLS LAST)
    TABLESPACE pg_default;