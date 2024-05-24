Feature: E-mail confirmation
    As a registered user, I need to confirm my e-mail address to start using the service

    Background:
            """
            The background for this feature is that a user has used the sign up form to register
            """
        Given a user visits the sign up page
        And the 'name' field is filled with 'Peter Rabbit'
        And the 'email' field is filled with 'peter@rabbit.com'
        And the 'password' field is filled with '^3aBa#z79^7jqz'
        And the 'password_confirm' field is filled with '^3aBa#z79^7jqz'
        And the form is submitted

    Scenario: Use the e-mail to confirm the registration
        Given a user successfully signed up
        And a user visits their e-mail inbox
        When the most recent e-mail is opened
        And a confirmation link is clicked
        Then the registration is confirmed


