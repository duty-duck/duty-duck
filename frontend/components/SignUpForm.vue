<script lang="ts" setup>
const state = ({
    firstName: "",
    lastName: "",
    email: "",
    password: "",
    orgName: ""
});

</script>
<template>
    <BForm validated novalidate>
        <div class="row mb-4">
            <h2 class="h5">Tell us about yourself</h2>
            <div class="col-lg-6">
                <BFormGroup id="firstNameGroup" label="First Name" label-for="firstNameInput"
                    description="Let us know your name" floating>
                    <BFormInput v-model="state.firstName" id="firstNameInput" placeholder="Enter your first name please"
                        required />
                </BFormGroup>
            </div>
            <div class="col-lg-6">
                <BFormGroup id="lastNameGroup" label="Last Name" label-for="lastNameInput" floating>
                    <BFormInput v-model="state.lastName" id="lastNameInput" placeholder="Enter your last name please"
                        required />
                </BFormGroup>
            </div>
        </div>
        <BFormGroup class="mb-3" id="emailGroup" label="E-mail address" label-for="emailInput"
            description="We won't share your e-mail with anyone" floating>
            <BFormInput v-model="state.email" id="emailInput" type="email" placeholder="Enter your email please"
                required />
        </BFormGroup>
        <BFormGroup class="mb-5" id="passwordGroup" label="Password" label-for="passwordInput" floating
            description="We recommend using a strong, random">
            <BFormInput v-model="state.password" id="passwordInput" type="password"
                placeholder="Enter your password please" required />
        </BFormGroup>
        <div class="mb-3">
            <h2 class="h5">Tell us about your oganization</h2>
            <p>
                After you sign up, you will be able to invite other members of your organization to collaborate. The
                name of
                your organization will show on e-mails and invoices.
            </p>
            <BFormGroup id="orgNameGroup" label="Your organization" label-for="orgInput"
                description="Don't worry, you will be able to rename your organization later" floating>
                <BFormInput v-model="state.orgName" id=" orgInput" placeholder="Enter your organization's anem"
                    required />
            </BFormGroup>
        </div>
        <div>
            <BButton type="submit" ">Sign up</BButton>
        </div>
    </BForm>
</template>