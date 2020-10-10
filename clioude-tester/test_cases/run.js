import { Selector } from 'testcafe'
import { MAIN_URL } from '../utils/const'

fixture('Normal Run Example')
    .page(MAIN_URL)

test('C++', async t => {
    await t.click('#trigger')
    await t.expect(Selector('#status').withText('Success').exists).ok()
})

test('Python', async t => {
    await t.click('#languages').click('#language-Python3')
    await t.click('#reset')
    await t.click('#trigger')
    await t.expect(Selector('#status').withText('Success').exists).ok()
})