fn update(world, entity) {
    let xform = get_component("Transform", world, entity);
    let vel = get_component("Velocity", world, entity);
    let dir = get_component("Direction", world, entity);
    
    let new_vel = new_vec2(0., 0.);
    if input::is_key_down(Key::W) {
      new_vel.y += -1.;
    }
    if input::is_key_down(Key::S) {
      new_vel.y += 1.;
    }
    if input::is_key_down(Key::A) {
      new_vel.x += -1.;
    }
    if input::is_key_down(Key::D) {
      new_vel.x += 1.;
    }

    let anim = get_component("Animation", world, entity);
    let anim_list = get_component("PlayerSpritesheets", world, entity);

    if new_vel.x == 0. && new_vel.y == 0. {
      anim.texture_handle = anim_list.idle;
      anim.active_animation = animation_name(dir, "IDLE");
    } else {
      dir = direction_from_velocity(new_vel);
      anim.texture_handle = anim_list.walk;
      anim.active_animation = animation_name(dir, "WALK");
      new_vel.normalize();
    }
    vel.direction = new_vel;

    xform.position.x += vel.direction.x * vel.speed * dt;
    xform.position.y += vel.direction.y * vel.speed * dt;

    set_component("Transform", world, entity, xform);
    set_component("Velocity", world, entity, vel);
    set_component("Direction", world, entity, dir);
    set_component("Animation", world, entity, anim);
}

// vim: ft=rhai
