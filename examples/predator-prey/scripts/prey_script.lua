energy = 100
age = 0
reproduction_cooldown = 0
next_offspring_id = 1

function update(current_time, msgs)
    age = age + 1
    -- Check if hunted
    local eaten = false
    for _, msg in ipairs(msgs) do
        if msg.kind == "hunt" then
            eaten = true
            break
        end
    end
    
    if eaten then
        self.destroy(self.id)
        return
    end
    
    -- Gain energy from grazing
    energy = energy + 5
    if energy > 150 then
        energy = 150
    end
    
    -- Death from age or starvation
    if age > 100 or energy <= 0 then
        self.destroy(self.id)
        return
    end
    
    -- Reproduction
    if reproduction_cooldown > 0 then
        reproduction_cooldown = reproduction_cooldown - 1
    end
    
    if energy > 100 and age > 10 and reproduction_cooldown == 0 then
        local offspring_id = "prey_" .. self.id .. "_" .. next_offspring_id
        next_offspring_id = next_offspring_id + 1
        energy = energy - 40
        reproduction_cooldown = 20
        self.spawn_entity(offspring_id, "prey_script", {energy = 80, age = 0, reproduction_cooldown = 10, next_offspring_id = 1})
    end
end

function get_state()
    return {energy = energy, age = age, reproduction_cooldown = reproduction_cooldown, next_offspring_id = next_offspring_id}
end

function set_state(state)
    energy = state.energy
    age = state.age
    reproduction_cooldown = state.reproduction_cooldown
    next_offspring_id = state.next_offspring_id or 1
end