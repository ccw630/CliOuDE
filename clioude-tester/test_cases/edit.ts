import { Selector } from 'testcafe'
import { MAIN_URL } from '../utils/const'

fixture('Autocomplete')
    .page(MAIN_URL)

test('Python', async t => {
    await t.click('#languages').click('#language-Python3')
    await t.click('#reset')
    await t.click('.view-line')
    await t.pressKey('end')
    for (let i = 0; i < 21; i++) await t.pressKey('backspace')
    await t.pressKey('i m p')
    await t.expect(Selector('.contents > .main').withText('import').exists).ok()
})