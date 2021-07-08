// Win rate percentage taking into account games played
// https://www.evanmiller.org/how-not-to-sort-by-average-rating.html
pub fn calc_win_rate(games_won: f32, games_played: f32) -> f32 {
    if games_played == 0.0 {
        return 0.0;
    }
    //4.265 == 99.999% confidence
    let confidence_inverse_norm_dist = 4.265;
    let phat = 1.0 * games_won / games_played;
    (phat + confidence_inverse_norm_dist * confidence_inverse_norm_dist / (2.0 * games_played)
        - confidence_inverse_norm_dist
            * f32::sqrt(
                (phat * (1.0 - phat)
                    + confidence_inverse_norm_dist * confidence_inverse_norm_dist
                        / (4.0 * games_played))
                    / games_played,
            ))
        / (1.0 + confidence_inverse_norm_dist * confidence_inverse_norm_dist / games_played)
}
