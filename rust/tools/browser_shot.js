// Run once in the browser tab at /_ref.html (see compare.md). Defines  __shot(x,y,z,yawDeg,pitchDeg,fov,name)
// which puts the game camera there (no people or items in the way), takes an 800 x 600 picture
// and saves it to the receiver on port 3999 as <name>.png.
window.__shot = async (x, y, z, yaw, pitch, fov, name) => {
  await __ev(`
    state='paused';
    document.querySelectorAll('body > *:not(canvas):not(script)').forEach(e=>e.style.display='none');
    if(window.__gal){scene.remove(window.__gal);window.__gal=null;}
    for(const c of chars.values()){if(c.mesh)c.mesh.visible=false;}
    for(const it of items.values()){if(it.mesh)it.mesh.visible=false;if(it.ring)it.ring.visible=false;}
    if(NPC.mesh)NPC.mesh.visible=false;
    camera.position.set(${x},${y},${z});
    camera.rotation.set(${pitch}*Math.PI/180,${yaw}*Math.PI/180,0);
    camera.fov=${fov};
    renderer.setPixelRatio(1);renderer.setSize(800,600,false);camera.aspect=800/600;camera.updateProjectionMatrix();
    renderer.render(scene,camera);
  `);
  const url = __ev('renderer.domElement.toDataURL("image/png")');
  await fetch('http://localhost:3999/?name=' + name + '.png', { method: 'POST', body: url });
  return name;
};
