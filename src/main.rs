
use rgraphics::*;

mod emulator;
mod assembler;

fn main() {

    let (program, mut event_loop) = Program::new();

    let mut chip8 = emulator::Chip8::new(&program);

    let asm = r#"

        jp .start
        
        .setup
            ld v0, 128      ; Ball sprite 2000

            ld v1, 128      ; paddle 2001
            ld v2, 128
            ld v3, 128
            ld v4, 128
            ld v5, 128

            ldi, 2000       ; Sprites stored at 2000
            dumpreg v5      ; Save sprites to memory
            ret

        .start
            call .setup

            ld v0, 16       ; Left paddle y
            ld v1, 16       ; Right paddle y

            ld v2, 0        ; Ball x
            ld v3, 0        ; Ball y
            ld v4, 0        ; Ball x direction (0 is right, 1 is left)
            ld v7, 0        ; Ball y direction (0 is up, 1 is down)

            ld v5, 0        ; Input
            ld v6, 0        ; Ball collision -- 0 is false, 1 is true

            ld v8, 0        ; left score
            ld v9, 0        ; right score

                            ;v10-v14 are general purpose
            
            call .reset_ball_left

            .game_loop
                call .main_loop
                jp .game_loop

            .end
            jp .end

        .main_loop
            call .update_paddles
            call .update_ball
            call .check_for_win
            call .draw_sprites
            ret

        .draw_score
            ldsprt v8
            ld v10, 16
            ld v11, 0
            drw v10, v11, 5

            ldsprt v9
            ld v10, 42
            drw v10, v11, 5
            ret


        .check_for_win
            sne v2, 0
            call .right_side_point
            
            sne v2, 63
            call .left_side_point
            ret

            .right_side_point
            call .reset_ball_left
            add v9, 1
            ret

            .left_side_point
            call .reset_ball_right
            add v8, 1
            ret

        .draw_sprites
            cls
            ld v15, 0

            ldi 2001
            ld v10, 62
            drw v10, v1, 5
            ld v10, 1
            drw v10, v0, 5
            ldi 2000
            drw v2, v3, 1
            ld v6, v15

            call .draw_score

            ret

        .update_paddles
            ld v10, 1
            sknp v10
            add v0, 1

            ld v10, 2
            ld v11, 1
            sknp v10
            sub v0, v11

            ld v10, 9
            sknp v10
            add v1, 1

            ld v10, 0
            ld v11, 1
            sknp v10
            sub v1, v11

            ret

        .update_ball

            ld v14, 1           ; ball speed

            sne v6, 1
            call .switch_ball_x_direction

            sne v4, 0
            add v2, v14

            sne v4, 1
            sub v2, v14

            ld v10, 0
            sne v3, v10
            call .switch_ball_y_direction

            ld v10, 31
            gte v3, v10
            sne v15, 1
            call .switch_ball_y_direction


            sne v7, 0
            add v3, v14

            sne v7, 1
            sub v3, v14

            ret


        .reset_ball_left

            rnd v10, 31
            ld v2, 8
            ld v3, v10
            ld v4, 0
            ret


        .reset_ball_right
            rnd v10, 31
            ld v2, 47
            ld v3, v10
            ld v4, 1
            ret

        .switch_ball_x_direction
            sne v4, 0
            jp .switch_ball_0_to_1

            ld v4, 0
            ret 

            .switch_ball_0_to_1
                ld v4, 1
                ret

        .switch_ball_y_direction
            sne v7, 0
            jp .switch_ball_y_0_to_1

            ld v7, 0
            ret

            .switch_ball_y_0_to_1
                ld v7, 1
                ret
                   

    "#;

    //chip8.load_rom_data(assembler::assemble(&asm));
    chip8.load_rom_from_file("C:/Users/lukes/Desktop/Code/Rust/chip8/roms/breakout.ch8");
    chip8.set_cycles_per_second(200);

    rgraphics::run(program, &mut event_loop, &mut |program| {
        chip8.clock(program);
    });
}