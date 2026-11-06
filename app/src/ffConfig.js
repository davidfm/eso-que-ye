// Modified by David Fernández, 2026. See git history for details.

// These parameters are all just copied from ff_config.rs, which provides a
//  detailed explanation as to the meaning of these values. It's simply
//  convenient for the frontend to be able to easily access some of these
//  global constants.
const ffConfig = {
    // eslint-disable-next-line no-undef
    FRONTEND_VERSION: process.env.PACKAGE_VERSION,
    RECORDING_TIME_LIMIT_MS: 30000,
    SAMPLE_RATE_DEFAULT: 48000,
    SAMPLE_RATE_MIN: 3952,
    SAMPLE_RATE_MAX: 66974,
    SPEC_WINDOW_SIZE: 1024,

    // Frontend only. Recording stops before the time limit once the best
    //  match is clear: a score of at least EARLY_STOP_MIN_SCORE, ahead of the
    //  second best tune by at least EARLY_STOP_MIN_MARGIN. Checks start at the
    //  old 10 s limit, so stopping early is never worse than before.
    EARLY_STOP_FIRST_CHECK_MS: 10000,
    EARLY_STOP_CHECK_INTERVAL_MS: 2500,
    EARLY_STOP_MIN_SCORE: 0.5,
    EARLY_STOP_MIN_MARGIN: 0.15,
};

export default ffConfig;