import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
const source = await readFile(new URL('./pointer-lock.js', import.meta.url), 'utf8');
const { installPointerLock } = await import('data:text/javascript;base64,' + Buffer.from(source).toString('base64'));
function surface() {
  const listeners = new Map();
  return {addEventListener(t,f){listeners.set(t,f)}, fire(t,e={}){listeners.get(t)?.(e)}};
}
const canvas = {...surface(), playing:false, getAttribute(){return String(this.playing)}, focus(){}, requestPointerLock(){requests++;}};
const button = surface();
const doc = {...surface(), getElementById(id){return id==='bbq'?canvas:button}, pointerLockElement:null,
 exitPointerLock(){this.pointerLockElement=null;this.fire('pointerlockchange')}};
let update, requests=0;
const win = {...surface(), MutationObserver:class {constructor(fn){update=fn} observe(){}}};
installPointerLock(doc,win);
assert.equal(button.hidden,true);
canvas.playing=true;update();assert.equal(button.hidden,false);assert.equal(requests,0);
button.fire('click',{preventDefault(){},stopPropagation(){}});assert.equal(requests,1);assert.equal(button.disabled,true);
doc.pointerLockElement=canvas;doc.fire('pointerlockchange');assert.equal(button.hidden,true);
doc.exitPointerLock();assert.equal(button.hidden,false);assert.equal(requests,1); // no automatic recapture
button.fire('click',{preventDefault(){},stopPropagation(){}});doc.fire('pointerlockerror');assert.equal(button.disabled,false);assert.match(button.textContent,/retry/);
doc.pointerLockElement=canvas;doc.fire('pointerlockchange');win.fire('blur');assert.equal(doc.pointerLockElement,null);assert.equal(button.hidden,false);
doc.pointerLockElement=canvas;canvas.playing=false;update();assert.equal(doc.pointerLockElement,null);assert.equal(button.hidden,true);
console.log('PASS: explicit capture, lock loss, retry, focus loss and menu release');
