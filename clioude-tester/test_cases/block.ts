import { Selector } from 'testcafe'

fixture('Blocked Run')
    .page('http://clioude.space')

test('os.system', async t => {
    await t.click('#languages').click('#language-Python3')
    await t.click('#reset')
    await t.click('.view-line')
    await t.pressKey('end')
    for (let i = 0; i < 21; i++) await t.pressKey('backspace')
    await t.pressKey('i m p o r t space o s enter o s . s y s t e m ( " l s " )')
    await t.click('#trigger')
    await t.expect(Selector('#status').withText('Runtime Error').exists).ok()
})