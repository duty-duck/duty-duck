Feature: Sign up
    As an anonymous user, I want to sign up so I can start using the service

    Background:
        Given a user visits the sign up page

    Scenario: Sign up attempt with a weak password
        When the 'name' field is filled with 'Jane Doe'
        And the 'email' field is filled with 'jane@e2etest.com'
        And the 'password' field is filled with 'a'
        And the 'password_confirm' field is filled with 'a'
        And the form is submitted
        Then the 'form' element should show 'Your password is too weak'

    Scenario: Sign up attempt with an invalid password confirmation
        When the 'name' field is filled with 'Jane Doe'
        And the 'email' field is filled with 'jane@e2etest.com'
        And the 'password' field is filled with 'PRB32ZZS#HBXu%'
        And the 'password_confirm' field is filled with '^3aBa#z79^7jqz'
        And the form is submitted
        Then the 'form' element should show 'The password confirmation does not match the password'

    Scenario: Sign up
        When the 'name' field is filled with 'Jane Doe'
        And the 'email' field is filled with 'jane@e2etest.com'
        And the 'password' field is filled with '^3aBa#z79^7jqz'
        And the 'password_confirm' field is filled with '^3aBa#z79^7jqz'
        And the form is submitted
        Then the '#auth-page-container' element should show 'Welcome!'
        And the '#auth-page-container' element should show 'A confirmation e-mail has been sent to your inbox. You must visit the provided URL to finish your registration'

    Scenario: Sign up for unconfirmed existing user
        When the 'name' field is filled with 'Jane Doe'
        And the 'email' field is filled with 'jane@e2etest.com'
        And the 'password' field is filled with '^3aBa#z79^7jqz'
        And the 'password_confirm' field is filled with '^3aBa#z79^7jqz'
        And the form is submitted
        Then the '#auth-page-container' element should show "You're almost there!"
        And the '#auth-page-container' element should show 'It seems that you have already signed up, but we need to confirm your e-mail address.'
