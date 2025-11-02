use abi::deck::calculate_hand_value;

#[test]
fn test_empty_hand() {
    let hand = vec![];
    assert_eq!(calculate_hand_value(&hand), 0);
}

#[test]
fn test_single_ace_from_each_suit() {
    // Spades Ace (card 1)
    assert_eq!(calculate_hand_value(&vec![1]), 11);
    // Hearts Ace (card 14)
    assert_eq!(calculate_hand_value(&vec![14]), 11);
    // Diamonds Ace (card 27)
    assert_eq!(calculate_hand_value(&vec![27]), 11);
    // Clubs Ace (card 40)
    assert_eq!(calculate_hand_value(&vec![40]), 11);
}

#[test]
fn test_two_aces() {
    // Two aces: one counted as 11, one as 1 = 12
    assert_eq!(calculate_hand_value(&vec![1, 14]), 12);
    assert_eq!(calculate_hand_value(&vec![27, 40]), 12);
}

#[test]
fn test_three_aces() {
    // Three aces: one counted as 11, two as 1 = 13
    assert_eq!(calculate_hand_value(&vec![1, 14, 27]), 13);
}

#[test]
fn test_four_aces() {
    // Four aces: one counted as 11, three as 1 = 14
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 40]), 14);
}

#[test]
fn test_five_aces() {
    // Five aces: one counted as 11, four as 1 = 15
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 40, 1]), 15);
}

#[test]
fn test_six_aces() {
    // Six aces: one counted as 11, five as 1 = 16
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 40, 1, 14]), 16);
}

#[test]
fn test_face_cards_all_suits() {
    // Spades Jack (11), Queen (12), King (13)
    assert_eq!(calculate_hand_value(&vec![11]), 10);
    assert_eq!(calculate_hand_value(&vec![12]), 10);
    assert_eq!(calculate_hand_value(&vec![13]), 10);

    // Hearts Jack (24), Queen (25), King (26)
    assert_eq!(calculate_hand_value(&vec![24]), 10);
    assert_eq!(calculate_hand_value(&vec![25]), 10);
    assert_eq!(calculate_hand_value(&vec![26]), 10);

    // Diamonds Jack (37), Queen (38), King (39)
    assert_eq!(calculate_hand_value(&vec![37]), 10);
    assert_eq!(calculate_hand_value(&vec![38]), 10);
    assert_eq!(calculate_hand_value(&vec![39]), 10);

    // Clubs Jack (50), Queen (51), King (52)
    assert_eq!(calculate_hand_value(&vec![50]), 10);
    assert_eq!(calculate_hand_value(&vec![51]), 10);
    assert_eq!(calculate_hand_value(&vec![52]), 10);
}

#[test]
fn test_number_cards() {
    // Spades 2-10 (cards 2-10)
    assert_eq!(calculate_hand_value(&vec![2]), 2);
    assert_eq!(calculate_hand_value(&vec![3]), 3);
    assert_eq!(calculate_hand_value(&vec![5]), 5);
    assert_eq!(calculate_hand_value(&vec![10]), 10);

    // Hearts 2-10 (cards 15-23)
    assert_eq!(calculate_hand_value(&vec![15]), 2);
    assert_eq!(calculate_hand_value(&vec![19]), 6);
    assert_eq!(calculate_hand_value(&vec![23]), 10);

    // Diamonds 2-10 (cards 28-36)
    assert_eq!(calculate_hand_value(&vec![28]), 2);
    assert_eq!(calculate_hand_value(&vec![32]), 6);
    assert_eq!(calculate_hand_value(&vec![36]), 10);

    // Clubs 2-10 (cards 41-49)
    assert_eq!(calculate_hand_value(&vec![41]), 2);
    assert_eq!(calculate_hand_value(&vec![45]), 6);
    assert_eq!(calculate_hand_value(&vec![49]), 10);
}

#[test]
fn test_blackjack() {
    // Ace + 10-value card = 21
    assert_eq!(calculate_hand_value(&vec![1, 10]), 21); // Ace + 10
    assert_eq!(calculate_hand_value(&vec![1, 11]), 21); // Ace + Jack
    assert_eq!(calculate_hand_value(&vec![14, 12]), 21); // Ace + Queen
    assert_eq!(calculate_hand_value(&vec![27, 13]), 21); // Ace + King
    assert_eq!(calculate_hand_value(&vec![40, 50]), 21); // Ace + Jack (different suits)
}

#[test]
fn test_ace_adjustment_to_prevent_bust() {
    // Ace + 9 + 5 = would be 25 with Ace as 11, but adjusts to 15
    assert_eq!(calculate_hand_value(&vec![1, 9, 5]), 15);

    // Ace + 10 + 5 = would be 26 with Ace as 11, but adjusts to 16
    assert_eq!(calculate_hand_value(&vec![14, 10, 5]), 16);

    // Two Aces + 9 = would be 31, adjusts to 21 (one Ace = 11, one = 1)
    assert_eq!(calculate_hand_value(&vec![1, 14, 9]), 21);
}

#[test]
fn test_multiple_aces_with_face_cards() {
    // Three Aces + King: 11+11+11+10=43 → adjust to 1+1+1+10 = 13
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 13]), 13);

    // Two Aces + King: 11+11+10=32 → adjust to 1+1+10 = 12
    assert_eq!(calculate_hand_value(&vec![1, 14, 13]), 12);

    // Ace + King = 11 + 10 = 21
    assert_eq!(calculate_hand_value(&vec![1, 13]), 21);
}

#[test]
fn test_soft_17() {
    // Ace + 6 = 17 (soft 17)
    assert_eq!(calculate_hand_value(&vec![1, 6]), 17);
}

#[test]
fn test_hard_17() {
    // 10 + 7 = 17 (hard 17)
    assert_eq!(calculate_hand_value(&vec![10, 7]), 17);
}

#[test]
fn test_bust_scenarios() {
    // 10 + 10 + 5 = 25 (bust)
    assert_eq!(calculate_hand_value(&vec![10, 23, 5]), 25);

    // King + Queen + Jack = 30 (bust)
    assert_eq!(calculate_hand_value(&vec![13, 25, 37]), 30);
}

#[test]
fn test_perfect_21_without_blackjack() {
    // 7 + 7 + 7 = 21
    assert_eq!(calculate_hand_value(&vec![7, 20, 33]), 21);

    // 10 + 6 + 5 = 21
    assert_eq!(calculate_hand_value(&vec![10, 19, 31]), 21);

    // 3 + 8 + 10 = 21
    assert_eq!(calculate_hand_value(&vec![3, 21, 36]), 21);
}

#[test]
fn test_complex_ace_scenarios() {
    // Ace + Ace + Ace + 8 = 11 + 1 + 1 + 8 = 21
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 8]), 21);

    // Ace + Ace + 9 = 11 + 1 + 9 = 21
    assert_eq!(calculate_hand_value(&vec![1, 14, 9]), 21);

    // Ace + Ace + Ace + Ace + 7 = 11 + 1 + 1 + 1 + 7 = 21
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 40, 7]), 21);
}

#[test]
fn test_eight_deck_scenario_multiple_same_cards() {
    // Simulate drawing multiple same-rank cards from 8 decks
    // Eight 2s = 16
    assert_eq!(calculate_hand_value(&vec![2, 15, 28, 41, 2, 15, 28, 41]), 16);

    // Eight Aces = 11 + 7*1 = 18
    assert_eq!(calculate_hand_value(&vec![1, 14, 27, 40, 1, 14, 27, 40]), 18);
}

#[test]
fn test_edge_case_all_low_cards() {
    // 2 + 2 + 2 + 2 + 2 = 10
    assert_eq!(calculate_hand_value(&vec![2, 15, 28, 41, 2]), 10);
}

#[test]
fn test_mixed_suits_same_ranks() {
    // Four 5s from different suits = 20
    assert_eq!(calculate_hand_value(&vec![5, 18, 31, 44]), 20);

    // Four Kings from different suits = 40
    assert_eq!(calculate_hand_value(&vec![13, 26, 39, 52]), 40);
}
