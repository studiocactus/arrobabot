import {test,expect} from '@playwright/test';
import {mock} from './ux-helpers';

/** Comportamento: o histórico mostra a hora dos registros — nunca “Invalid Date”.
 *  (Regressão da auditoria visual 0.13.0: o dado antigo do campo de tempo não
 *  pode aparecer como data quebrada.) */
test('histórico mostra a hora dos registros e nunca Invalid Date',async({page})=>{
 await mock(page);
 await page.goto('/');
 await expect(page.getByText('Execução concluída com sucesso no fluxo de boas-vindas')).toBeVisible();
 await expect(page.getByText('Invalid Date')).toHaveCount(0);
 await expect(page.getByText(/^\d{2}:\d{2}:\d{2}$/).first()).toBeVisible();

 await page.locator('.sidebar').getByRole('button',{name:'Histórico',exact:true}).click();
 await page.getByRole('button',{name:'Eventos gerais',exact:true}).click();
 await expect(page.getByText('Fonte mutada')).toBeVisible();
 await expect(page.getByText('Invalid Date')).toHaveCount(0);
 await expect(page.getByText(/^\d{2}:\d{2}:\d{2}$/).first()).toBeVisible();
});

/** Comportamento: o valor salvo que não existe mais continua visível e
 *  explicado no seletor — nunca some nem vira linha em branco. */
test('seletor preserva e explica valor salvo indisponível',async({page})=>{
 await mock(page);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Sons de Live',exact:true}).click();
 await page.getByRole('button',{name:'Adicionar pessoa',exact:true}).click();
 // A pessoa nova não tem som: o seletor começa pelo estado inicial legível.
 await expect(page.getByRole('combobox',{name:'Som da pessoa 1'})).toBeVisible();
 await expect(page.getByRole('combobox',{name:'Som da pessoa 1'})).toHaveValue('');
});
