-- This file should undo anything in `up.sql`
DROP VIEW IF EXISTS legacy_migration_v1.move_resources;
DROP VIEW IF EXISTS legacy_migration_v1.address_version_from_move_resources;
DROP VIEW IF EXISTS legacy_migration_v1.coin_activities;
DROP VIEW IF EXISTS legacy_migration_v1.coin_balances;
DROP VIEW IF EXISTS legacy_migration_v1.coin_infos;
DROP VIEW IF EXISTS legacy_migration_v1.current_coin_balances;
DROP VIEW IF EXISTS legacy_migration_v1.token_activities;
DROP VIEW IF EXISTS legacy_migration_v1.token_ownerships;
DROP VIEW IF EXISTS legacy_migration_v1.current_token_ownerships;
DROP VIEW IF EXISTS legacy_migration_v1.tokens;
DROP VIEW IF EXISTS legacy_migration_v1.token_datas;
DROP VIEW IF EXISTS legacy_migration_v1.current_token_datas;
DROP VIEW IF EXISTS legacy_migration_v1.collection_datas;
DROP VIEW IF EXISTS legacy_migration_v1.current_ans_primary_name;
DROP VIEW IF EXISTS legacy_migration_v1.current_ans_lookup;
DROP INDEX IF EXISTS lm1_cv_ci_tv_index;
DROP INDEX IF EXISTS lm1_ta_tdih_pv_index;
DROP INDEX IF EXISTS lm1_cb_tv_oa_ct_index;
DROP INDEX IF EXISTS lm1_curr_to_oa_tt_ltv_index;
DROP INDEX IF EXISTS lm1_ccb_ct_a_index;
DROP INDEX IF EXISTS lm1_tdv_tdi_tv_index;
DROP INDEX IF EXISTS lm1_curr_to_oa_tt_am_ltv_index;
DROP INDEX IF EXISTS lm1_ca_ct_a_index;
DROP INDEX IF EXISTS lm1_ca_ct_at_a_index;
DROP INDEX IF EXISTS lm1_ca_oa_ct_at_index;
DROP INDEX IF EXISTS lm1_ca_oa_igf_index;
DROP INDEX IF EXISTS lm1_ans_d_s_et_index;
DROP INDEX IF EXISTS lm1_ans_ra_et_index;
DROP SCHEMA IF EXISTS legacy_migration_v1;