import unittest

from check_speech import word_error_rate


class SpeechChecks(unittest.TestCase):
    def test_case_and_punctuation_do_not_change_the_score(self):
        self.assertEqual(word_error_rate("Guten Morgen!", "guten morgen.")["wordEdits"], 0)

    def test_unicode_spelling_is_normalised(self):
        self.assertEqual(word_error_rate("Straße Café", "STRASSE Cafe\u0301")["wordEdits"], 0)

    def test_dropped_words_are_measured(self):
        result = word_error_rate("eins zwei drei vier", "eins vier")
        self.assertEqual(result["wordErrorPercent"], 50)

    def test_added_and_substituted_words_are_measured(self):
        result = word_error_rate("eins zwei", "eins drei vier")
        self.assertEqual(result["wordEdits"], 2)

    def test_empty_reference_is_not_a_valid_perfect_score(self):
        with self.assertRaises(ValueError):
            word_error_rate("", "")


if __name__ == "__main__":
    unittest.main()
