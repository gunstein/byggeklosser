
//use cgmath::prelude::*;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;

#[derive(Debug)]
pub struct MousePicker {
    
}

impl MousePicker{
    pub fn get_model_coordinates_for_voxel_under_mouse( window_size: &winit::dpi::PhysicalSize<u32>, mouse_device_coord: &winit::dpi::PhysicalPosition<f64>, 
                                                camera: &crate::camera::Camera, projection: &crate::camera::Projection, model: &crate::model::Model) -> Option<cgmath::Vector3<i32>>
    {
        //https://antongerdelan.net/opengl/raycasting.html
        // Step 1: 3d Normalised Device Coordinates
        let x = (2.0 * mouse_device_coord.x) / window_size.width as f64 - 1.0;
        let y = 1.0 - (2.0 * mouse_device_coord.y) / window_size.height as f64;
        let z = 1.0;
        let ray_nds : cgmath::Vector3<f32> = cgmath::Vector3::new(x as f32, y as f32, z);

        println!("x {:?}", x);
        println!("y {:?}", y);
        println!("ray_nds {:?}", ray_nds);

        //Step 2: 4d Homogeneous Clip Coordinates
        let ray_clip : cgmath::Vector4<f32> = cgmath::Vector4::new(ray_nds.x, ray_nds.y, -1.0, 1.0);

        //Step 3: 4d Eye (Camera) Coordinates
        //vec4 ray_eye = inverse(projection_matrix) * ray_clip;
        let ray_eye = projection.calc_matrix().invert().unwrap() * ray_clip;
        let ray_eye = cgmath::Vector4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
        println!("ray_eye {:?}", ray_eye);

        //Step 4: 4d World Coordinates
        //vec3 ray_wor = (inverse(view_matrix) * ray_eye).xyz;
        let ray_wor_v4 = camera.calc_matrix().invert().unwrap() * ray_eye;
        let ray_wor : cgmath::Vector3<f32> = cgmath::Vector3::new(ray_wor_v4.x, ray_wor_v4.y, ray_wor_v4.z);
        println!("ray_wor {:?}", ray_wor);

        // don't forget to normalise the vector at some point
        let ray_wor = ray_wor.normalize();
        println!("ray_wor_normalized {:?}", ray_wor);

        //Use ray_wor to find right voxel
        //J. Amanatides, A. Woo. A Fast Voxel Traversal Algorithm for Ray Tracing.
        //Based on this implementation:
        //https://github.com/francisengelmann/fast_voxel_traversal/blob/master/main.cpp
        const MAX_DISTANCE : u32 = 20;
        println!("camera.position.x {:?}", camera.position.x);
        println!("camera.position.y {:?}", camera.position.y);
        println!("camera.position.z {:?}", camera.position.z);

        let ray_start : cgmath::Vector3<f32> = cgmath::Vector3::new(camera.position.x, camera.position.y, camera.position.z);
        let mut current_block : cgmath::Vector3<i32> = cgmath::Vector3::new(camera.position.x.floor() as i32, camera.position.y.floor() as i32, camera.position.z.floor() as i32);
        //let ray_start = current_block.clone();
        println!("ray_start {:?}", ray_start);

        // In which direction the voxel ids are incremented.
        let step_x = if ray_wor[0] >= 0.0 {1} else {-1};
        let step_y = if ray_wor[1] >= 0.0 {1} else {-1};
        let step_z = if ray_wor[2] >= 0.0 {1} else {-1};
        println!("step_x {:?}", step_x);
        println!("step_y {:?}", step_y);
        println!("step_z {:?}", step_z);

        // Distance along the ray to the next voxel border from the current position (tMaxX, tMaxY, tMaxZ).
        let next_block_boundary_x = current_block[0]+step_x;
        let next_block_boundary_y = current_block[1]+step_y;
        let next_block_boundary_z = current_block[2]+step_z;

        // tMaxX, tMaxY, tMaxZ -- distance until next intersection with voxel-border
        // the value of t at which the ray crosses the first vertical voxel boundary
        let mut t_max_x = if ray_wor[0] != 0.0 {(next_block_boundary_x as f32 - ray_start[0])/ray_wor[0]} else {std::f32::MAX};
        let mut t_max_y = if ray_wor[1] != 0.0 {(next_block_boundary_y as f32 - ray_start[1])/ray_wor[1]} else {std::f32::MAX};
        let mut t_max_z = if ray_wor[2] != 0.0 {(next_block_boundary_z as f32 - ray_start[2])/ray_wor[2]} else {std::f32::MAX};

        // tDeltaX, tDeltaY, tDeltaZ --
        // how far along the ray we must move for the horizontal component to equal the width of a voxel
        // the direction in which we traverse the grid
        // can only be FLT_MAX if we never go in that direction
        let t_delta_x = if ray_wor[0]!=0.0 {1.0/ray_wor[0]*step_x as f32} else {std::f32::MAX};
        let t_delta_y = if ray_wor[1]!=0.0 {1.0/ray_wor[1]*step_y as f32} else {std::f32::MAX};
        let t_delta_z = if ray_wor[2]!=0.0 {1.0/ray_wor[2]*step_z as f32} else {std::f32::MAX};

        println!("t_delta_x {:?}", t_delta_x);
        println!("t_delta_y {:?}", t_delta_y);
        println!("t_delta_z {:?}", t_delta_z);  
        
        let mut diff : cgmath::Vector3<i32> = cgmath::Vector3::new(0, 0, 0);
        let mut neg_ray : bool = false;
        if ray_wor[0]<0.0 { diff[0]-=1; neg_ray=true; }
        if ray_wor[1]<0.0 { diff[1]-=1; neg_ray=true; }
        if ray_wor[2]<0.0 { diff[2]-=1; neg_ray=true; }

        if neg_ray {
          current_block+=diff;
        }

        let mut counter : u32 = 0;
        let mut found : bool = false;
        //let mut search_block : Option<&crate::model::Block> = None;
        let mut result : Option<cgmath::Vector3<i32>> = None;
        while found == false && counter < MAX_DISTANCE{
            if t_max_x < t_max_y {
              if t_max_x < t_max_z {
                current_block[0] += step_x;
                t_max_x += t_delta_x;
              } else {
                current_block[2] += step_z;
                t_max_z += t_delta_z;
              }
            } else {
              if t_max_y < t_max_z {
                current_block[1] += step_y;
                t_max_y += t_delta_y;
              } else {
                current_block[2] += step_z;
                t_max_z += t_delta_z;
              }
            }
            println!("t_max_x {:?}", t_max_x);
            println!("t_max_y {:?}", t_max_y);
            println!("t_max_z {:?}", t_max_z);    
            counter += 1;
            println!("current_block {:?}", current_block);
            let search_block = model.world.GetBlockFromGlobalAddress(current_block.x as f64, current_block.y as f64, current_block.z as f64);
            if search_block.is_some(){
              println!("FOUND!");
                found = true;
                result = Some(cgmath::Vector3::new(current_block.x, current_block.y, current_block.z) );
            }
        }

        result
    }

}