import { expect, test, type ConsoleMessage, type Page } from '@playwright/test'

const TOKEN = 'e2e-token'

function collectErrors(page: Page): string[] {
  const errors: string[] = []
  page.on('console', (message: ConsoleMessage) => {
    if (message.type() === 'error') errors.push(message.text())
  })
  page.on('pageerror', error => errors.push(error.message))
  return errors
}

test.beforeEach(async ({ page }) => {
  await page.goto(`/?token=${TOKEN}`)
  await expect(page).toHaveTitle(/twatch/i)
})

test('boots the console and authenticates the API', async ({ page }) => {
  const errors = collectErrors(page)
  await expect(page.getByRole('heading', { name: 'Torrents, in motion.' })).toBeVisible()
  await expect(page.getByRole('navigation', { name: 'Main navigation' }).first()).toBeVisible()

  const response = await page.request.get('/api/bootstrap')
  expect(response.status()).toBe(200)
  const body = (await response.json()) as { version: string }
  expect(body.version).toBeTruthy()
  expect(errors).toEqual([])
})

test('rejects unauthenticated API requests', async ({ playwright }) => {
  const context = await playwright.request.newContext({
    baseURL: 'http://127.0.0.1:4173',
  })
  const response = await context.get('/api/bootstrap')
  expect(response.status()).toBe(401)
  await context.dispose()
})

test('navigates between the main pages', async ({ page }) => {
  const nav = page.getByRole('navigation', { name: 'Main navigation' }).first()
  await nav.getByRole('link', { name: 'Downloads' }).click()
  await expect(page).toHaveURL(/\/downloads$/)
  await expect(page.getByRole('heading', { name: 'Downloads in flight' })).toBeVisible()

  await nav.getByRole('link', { name: 'History' }).click()
  await expect(page).toHaveURL(/\/history$/)
  await expect(page.getByRole('heading', { name: 'Everything you have opened' })).toBeVisible()

  await nav.getByRole('link', { name: 'Settings' }).click()
  await expect(page).toHaveURL(/\/settings$/)
  await expect(page.getByRole('heading', { name: 'Shape your console' })).toBeVisible()
})

test('renders the mobile navigation', async ({ page, isMobile }) => {
  test.skip(!isMobile, 'mobile project only')
  await expect(page.getByRole('navigation', { name: 'Main navigation' }).last()).toBeVisible()
})
