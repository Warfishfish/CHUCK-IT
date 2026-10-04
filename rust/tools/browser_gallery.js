// Run in the browser tab at /_ref.html (see compare.md). Lines the items up against the sky and
// saves an 800 x 600 picture to the receiver on port 3999 as browser_gallery.png.
// Use `__ev(...)` to run code inside the game; `calib` swaps the items for one white ball.
(async (calib) => {
  await __ev(`
    state='paused';
    document.querySelectorAll('body > *:not(canvas):not(script)').forEach(e=>e.style.display='none');
    clouds.forEach(c=>c.visible=false);
    if(window.__gal)scene.remove(window.__gal);
    window.__gal=new THREE.Group();scene.add(__gal);
    ${calib
      ? `const s=new THREE.Mesh(new THREE.SphereGeometry(.8,64,48),new THREE.MeshLambertMaterial({color:0xffffff}));s.position.set(0,30,-1);s.castShadow=false;s.receiveShadow=false;__gal.add(s);`
      : `['teddy','stubby','gnome','steak','fish','noodle','dildo'].forEach((k,i)=>{const o=MAKERS[k]();o.position.set((i-3)*.55,30.35,-1);__gal.add(o);});
         ['dildoMini','dildoJumbo','dildoGold'].forEach((v,i)=>{const o=MAKERS.dildo(v);o.position.set((i-1)*.6,29.7,-1);__gal.add(o);});`}
    camera.position.set(0,30,2);camera.rotation.set(0,0,0);camera.fov=45;
    renderer.setPixelRatio(1);renderer.setSize(800,600,false);camera.aspect=800/600;camera.updateProjectionMatrix();
    renderer.render(scene,camera);
  `);
  const url = __ev('renderer.domElement.toDataURL("image/png")');
  await fetch('http://localhost:3999/?name=browser_' + (calib ? 'calib' : 'gallery') + '.png', { method: 'POST', body: url });
  return url.length;
})(false)
