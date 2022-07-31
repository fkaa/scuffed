<script setup lang="ts">
import { ref, reactive, computed } from "vue"

import InputText from "../../components/form/InputText.vue"
import { useFormValidation, minLength, required, sameAs } from "../../bin/validation"

const open = ref(false)
const loading = ref(false)
/**
 * User password validation
 */

const passForm = reactive({
  old: "",
  new1: "",
  new2: ""
})

const passIsEmpty = computed(
  () => passForm.old.length === 0 || passForm.new1.length === 0 || passForm.new2.length === 0
)

const passRules = computed(() => ({
  old: {
    required
  },
  new1: {
    required,
    minLenght: minLength(8)
  },
  new2: {
    required,
    sameAs: sameAs(passForm.new1)
  }
}))

const passValidation = useFormValidation(passForm, passRules, { autoclear: true })

async function savePassword() {
  // addLoading("password")
  loading.value = true

  passValidation
    .validate()
    .then(() => {
      // Submit here
    })
    .finally(() => {
      loading.value = false
    })
}
</script>

<template>
  <div class="account">
    <h2>Update password</h2>

    <form @submit.prevent="savePassword">
      <input id="username" style="display: none" type="text" name="fakeusernameremembered" />
      <input id="password" style="display: none" type="password" name="fakepasswordremembered" />
      <InputText
        type="password"
        :error="passValidation.errors.old"
        v-model:value="passForm.old"
        label="Old password"
        placeholder="Your current password"
      />
      <InputText
        type="password"
        :error="passValidation.errors.new1"
        v-model:value="passForm.new1"
        label="New password"
        placeholder="Your new password"
      />
      <InputText
        type="password"
        :error="passValidation.errors.new2"
        v-model:value="passForm.new2"
        label="Confirm password"
        placeholder="Confirm your new password"
      />

      <button style="margin-top: 20px" @click.prevent="savePassword" class="button btn-small">Save</button>
    </form>
  </div>
</template>
